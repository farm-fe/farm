"""Callable read-only release planning and evidence verification entrypoint.

No command in this file uploads packages, creates Git refs or reads publication
credentials. PRs can plan; live evidence reads are restricted to main Actions.
"""
# cspell:words argparse importlib urllib finalizer postpublish
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import urllib.parse


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(file))
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    return module


evidence = load("release_entry_evidence", "incremental-chain-evidence.py")
execution = load("release_entry_execution", "ci-execution-evidence.py")
retry = load("release_retry_safety", "release-retry-safety.py")
chain = evidence.chain_plan
check = evidence.check
WORKFLOW = ".github/workflows/incremental-release.yaml"
PRODUCER = "Build independent release candidates"
CONSUMER = "Verify independent release public consumers"
PUBLISHER = "Publish original npm archives and record every result"


def original_attempt(transport, run_id, source_sha, current_attempt, *, proof_provider=None):
    check(type(current_attempt) is int and 1 <= current_attempt <= 20, 'Invalid current release attempt')
    prefix = 'farm-release-candidates-' + source_sha + '-'; matches = []
    # Native products can exceed three artifact pages. Exact transaction names
    # keep the search bounded independently of native owner/platform inventory.
    for attempt in range(1, current_attempt + 1):
        name = prefix + str(attempt)
        response = transport.get('/actions/runs/' + str(run_id) + '/artifacts?per_page=2&name=' + name)
        rows = response['artifacts']
        check(response.get('total_count') == len(rows) and len(rows) <= 1 and all(row['name'] == name for row in rows),
              'Frozen release artifact lookup is incomplete or ambiguous')
        matches.extend(rows)
    if not matches:
        if current_attempt > 1:
            proof = retry.verify_unfrozen_retry(transport.get, run_id=run_id, source_sha=source_sha, current_attempt=current_attempt,
                workflow=WORKFLOW, producer=PRODUCER, freeze_step='Freeze independent release candidates',
                publisher=PUBLISHER, events=['push','workflow_dispatch'])
            print(json.dumps({'unfrozenRetryProof': proof}), flush=True)
        return current_attempt
    check(len(matches) == 1, 'Multiple frozen candidates cannot form one release transaction')
    artifact = matches[0]; attempt = int(artifact['name'][len(prefix):])
    check(artifact.get('expired') is False and 0 < attempt <= current_attempt and artifact['workflow_run']['id'] == run_id,
          'Original frozen release archive expired or differs; do not rebuild recovery')
    run = transport.get('/actions/runs/' + str(run_id) + '/attempts/' + str(attempt))
    check(run['id'] == run_id and run['run_attempt'] == attempt and run['path'] == WORKFLOW
          and run['head_sha'] == source_sha and run['head_branch'] == 'main', 'Original release producer source differs')
    commit=transport.get('/git/commits/'+source_sha)
    provenance=load('original_frozen_provenance','release-provenance.py')
    metadata,origin,producer=provenance.find_frozen_binding(transport,
        {'sha':source_sha,'tree':commit['tree']['sha']},'frozen',run_id,current_attempt,artifact['id'],'candidates',proof_provider=proof_provider)
    check(origin['run_attempt']==attempt and metadata['id']==artifact['id'], 'Original frozen producer attempt differs')

    return attempt


def source_plan(source, baseline=None, registry_snapshot=None):
    family = json.loads(source["files"]["docs/releases/v2-candidate.json"])
    check(baseline is None or baseline['ledger'].get('publicationScope') == 'npm', 'A mixed Cargo publication baseline cannot enter npm-only release planning')
    abi=re.search(r'^pub const VERSION: &str = "([0-9]+\.[0-9]+\.[0-9]+)";',source['files']['crates/core/src/lib.rs'].decode('utf-8'),re.M)
    check(abi is not None and family['abi']==abi[1],'Candidate Farm ABI differs from actual source declaration')
    prior=baseline['source'] if baseline else None
    if registry_snapshot is not None:
        selector=load('release_registry_selection','release-registry-selection.py')
        result=selector.plan(source,family,registry_snapshot)
    else:
        result=chain.plan_chain("release", {"sha": source["sha"], "tree": source["tree"]}, source["files"], family, [],
                                baseline=baseline['ledger'] if baseline else None,baseline_files=prior['files'] if prior else None,
                                anchor=baseline['anchor'] if baseline else None,source_modes=source["modes"],publication_scope="npm",baseline_modes=prior['modes'] if prior else None)
    nodes, issues = chain.repository_graph(source['files'])
    check(not issues, 'Release source dependency graph is invalid')
    if result.get('selectionPolicy') == 'registry-exact-version-v2':
        check(baseline is None, 'Version-presence planning cannot use a historical baseline')
        return result
    # JS build recipes execute Farm. Their native compiler is a source-built
    # preparation dependency, never a historical binary restore.
    build_nodes = set(result['buildNodes'])
    js_required = any(row['name'] in {key[4:] for key in result['build'] if key.startswith('npm:')}
                      and not row.get('owner') and not row['manifestPath'].startswith('rust-plugins/')
                      for row in result['fullFamily'] if row['kind'] == 'npm')
    pending = ['npm:packages/core'] if js_required else []
    while pending:
        key = pending.pop()
        if key in build_nodes: continue
        check(key in nodes, 'Release preparation node missing')
        build_nodes.add(key); pending.extend(nodes[key]['preparationDeps'])
    result['buildNodes'] = sorted(build_nodes)
    result['buildEdges'] = {key: sorted(nodes.get(key, {}).get('preparationDeps', set()) & build_nodes) for key in sorted(build_nodes)}
    native_names = {row['owner'] for row in family['npm']['payloads']}
    selected_names = {key[4:] for key in result['build'] if key.startswith('npm:')}
    result['preparationNativeOwners'] = sorted({row['name'] for row in family['npm']['primary']
        if row['name'] in native_names and ('npm:' + str(Path(row['manifestPath']).parent).replace('\\','/') in build_nodes or row['name'] in selected_names)})
    if baseline:
        result['baselineBinding']=baseline['transaction']
        # Registry URLs locate immutable bytes; they are not transaction or
        # build identity. Re-read their current value when restoring and verify
        # the frozen archive digest before use.
        result['baselinePackages']={key:{field:value for field,value in row.items() if field not in ['tarball','url','resolved']}
                                    for key,row in baseline['ledger']['packages'].items()}
    result['planHash']=chain.digest({key:value for key,value in result.items() if key!='planHash'})
    return result

def published_baseline(transport,transaction=None,read_source=evidence.source_reader):
    if transaction is not None:chain.exact(transaction,64)
    reader=load('release_completed_baseline','incremental-release-baseline.py')
    return reader.ReleaseBaselineReader(transport,evidence.registry_module.RegistryReadOnly(),read_source).lookup(transaction)

def release_checkpoint(manifest,proof):
    transport=actions_transport()
    check(os.environ.get('GITHUB_JOB')=='finalize','Only the trusted finalizer creates a durable release receipt')
    baseline=published_baseline(transport,manifest['baseline']) if manifest.get('baseline') is not None and manifest.get('registrySnapshot') is None else None
    source=evidence.source_reader(manifest['sourceSHA'])
    module=load('release_checkpoint_writer','incremental-release-checkpoint.py')
    value=module.create(manifest,source,baseline,proof,transport.get('')['id'])
    return module.encode(value)

def verify_release_checkpoint(manifest,release):
    transport=actions_transport()
    check(os.environ.get('GITHUB_JOB')=='finalize','Only the trusted finalizer confirms durable publication receipts')
    source=evidence.source_reader(manifest['sourceSHA'])
    module=load('release_checkpoint_confirmation','incremental-release-checkpoint.py')
    value=module.decode(release,manifest,source,transport.get('')['id'])
    return {'manifestHash':chain.digest(manifest),'checkpointHash':chain.digest(value),'sourceSHA':source['sha'],'sourceTree':source['tree']}


def strict_archive(directory, row):
    name = row.get("archive")
    check(isinstance(name, str) and re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]{0,179}", name), "Archive must be a canonical basename")
    root = Path(directory).resolve(strict=True)
    file = root / name
    check(not file.is_symlink() and file.resolve(strict=True).parent == root and file.is_file(), "Release archive escapes its sealed directory")
    size = file.stat().st_size
    check(0 < size <= 100 * 1024 * 1024 and size == row.get("archiveBytes"), "Release archive size differs")
    digest = hashlib.sha256()
    with file.open("rb") as stream:
        while block := stream.read(1024 * 1024):
            digest.update(block)
    check(digest.hexdigest() == row.get("archiveSha256"), "Release archive bytes differ")


def archive_integrity_mode(manifest):
    # This value is covered by the frozen manifest hash and signed seal. It is
    # never a substitute for verifying the source, seal and every archive byte.
    mode = manifest.get('releaseValidation')
    check(mode in [None, 'archive-integrity-v1'], 'Unknown frozen release validation policy')
    return mode == 'archive-integrity-v1'


def version_presence_mode(manifest):
    snapshot = manifest.get('registrySnapshot')
    presence = isinstance(snapshot, dict) and snapshot.get('schemaVersion') == 2
    if presence or manifest.get('selectionPolicy') == 'registry-exact-version-v2':
        check(presence and manifest.get('selectionPolicy') == 'registry-exact-version-v2', 'Version-presence manifest policy differs')
        check(manifest.get('baseline') is None and archive_integrity_mode(manifest), 'Version-presence manifest cannot use historical state or a different validation policy')
    return presence


def verify_presence_dispositions(proof, manifest, plan, family, records):
    check(not plan['registryUnknown'] and not manifest['buildFailures'], 'Complete version-presence proof cannot hide unknown versions or build failures')
    dispositions = proof.get('dispositions')
    check(isinstance(dispositions, dict) and set(dispositions) == set(family), 'Public version-presence disposition coverage differs')
    hashes = {}
    selector = load('verify_presence_dispositions', 'release-registry-selection.py')
    for identity, row in family.items():
        actual = dispositions[identity]
        check(isinstance(actual, dict) and actual.get('name') == row['name'] and actual.get('version') == row['version'], 'Public disposition version differs')
        selector.validate_observed(row, {**actual, 'status': 'published'}, 2)
        check(actual.get('latestVerified') is True and type(actual.get('uploadAttempted')) is bool, 'Public disposition latest or upload proof missing')
        origin = actual.get('originKind')
        observed = plan['registrySnapshot']['packages'][identity]
        if observed['status'] == 'published':
            check(origin == 'registry' and actual['uploadAttempted'] is False, 'Retained registry origin differs')
            check(actual.get('evidenceKind') == 'registry-metadata' and actual.get('registryBytesVerified') is False, 'Retained registry proof is metadata-only')
            for key in ['metadata', 'integrity']:
                check(actual.get(key) == observed.get(key), 'Retained registry observation differs')
        else:
            check(observed['status'] == 'missing' and identity in records and origin in ['source', 'registry-race'], 'Missing-version disposition origin differs')
            if origin == 'source':
                check(actual['uploadAttempted'] is True, 'New source disposition requires current upload provenance')
                check(actual.get('evidenceKind') == 'registry-archive' and actual.get('registryBytesVerified') is True, 'New source disposition requires verified archive bytes')
                check(actual['archiveSha256'] == records[identity]['archiveSha256'] and actual['archiveBytes'] == records[identity]['archiveBytes'], 'New source archive disposition differs')
            else:
                check(actual['uploadAttempted'] is False, 'Registry-race origin cannot claim an upload')
                check(actual.get('evidenceKind') == 'registry-metadata' and actual.get('registryBytesVerified') is False, 'Registry-race proof is metadata-only')
        hashes[identity] = actual.get('archiveSha256') if origin == 'source' else None
    check(proof.get('packages') == hashes, 'Public version-presence package coverage differs')
    return dispositions


def verify_current_presence(dispositions, family):
    """Finalization independently rereads identities, tags and required closure."""
    selector = load('verify_current_presence', 'release-registry-selection.py')
    registry = selector.Registry()
    metadata = {}
    for identity, row in sorted(family.items()):
        expected = dispositions[identity]
        archive = expected['originKind'] == 'source'
        actual = registry.lookup(row, verify_archive=archive)
        selector.validate_observed(row, actual, 2)
        check(actual.get('status') == 'published', 'Public version disappeared before finalization')
        fields = ['name', 'version', 'evidenceKind', 'metadata', 'integrity']
        if archive:
            fields += ['archiveSha256', 'archiveBytes']
        check(all(actual.get(field) == expected.get(field) for field in fields), 'Public version observation changed before finalization')
        metadata[row['name']] = actual['metadata']
        latest = registry.reader.json('https://registry.npmjs.org/' + urllib.parse.quote(row['name'], safe='') + '/latest')
        check(latest.get('name') == row['name'], 'Public latest package identity differs')
        def version(value):
            check(isinstance(value, str) and re.fullmatch(r'\d+\.\d+\.\d+', value), 'Exact stable latest version required')
            return tuple(map(int, value.split('.')))
        check(version(latest.get('version')) >= version(row['version']), 'Public latest tag is behind selected stable release')
    publication = load('verify_current_presence_dependencies', 'release-npm-best-effort.py')
    for peers_only in [False, True]:
        dependencies = publication.required_dependencies(list(family.values()), metadata, peers_only=peers_only)
        check(set(dependencies) == set(metadata) and all(required <= set(metadata) for required in dependencies.values()), 'Public required dependency closure incomplete')


def public_scope(manifest):
    return 'registry' if archive_integrity_mode(manifest) else 'public'


def public_integrity_verified(proof, manifest):
    if version_presence_mode(manifest):
        return (proof.get('kind') == 'release-version-presence-v1' and proof.get('registryMetadataVerified') is True
                and proof.get('testsExecuted') is False and proof.get('registryInstall') is False
                and proof.get('nativeABI') is False and proof.get('latestTargets') is True)
    if archive_integrity_mode(manifest):
        return (proof.get('kind') == 'release-byte-integrity-v1' and proof.get('registryBytesVerified') is True and proof.get('testsExecuted') is False
                and proof.get('registryInstall') is False and proof.get('nativeABI') is False and proof.get('latestTargets') is True)
    return proof.get('kind') != 'release-byte-integrity-v1' and all(proof.get(key) is True for key in ['registryInstall', 'nativeABI', 'latestTargets'])


class ReleaseVerifier:
    def __init__(self, transport, read_source=evidence.source_reader, *, receipt_provider=None):
        self.transport, self.read_source = transport, read_source
        # Explicit Python-only offline seam. Production CLI never supplies it.
        self.receipt_provider=receipt_provider

    def verify_archive(self, directory, row):
        strict_archive(directory, row)
        return True

    def receipt(self, scope, run, source, manifest=None):
        if self.receipt_provider is not None:return self.receipt_provider(scope,run,source)
        provenance=load('release_entry_provenance','release-provenance.py')
        if scope=='frozen':return provenance.frozen_json(self.transport,run,source,'seal')
        result=provenance.signed_json(self.transport,run,source,public_scope(manifest),'public')
        if archive_integrity_mode(manifest):
            check(result[0].get('kind') == ('release-version-presence-v1' if version_presence_mode(manifest) else 'release-byte-integrity-v1'), 'Registry proof kind differs')
            return result[:4]
        check(isinstance(manifest,dict),'Public proof requires the frozen manifest')
        consumers=load('release_entry_consumers','release-consumer-bindings.py')
        for platform,output in provenance.consumer_outputs(result[5],'candidate').items():
            consumers.receipt(self.transport,run,manifest,'candidate',platform,output)
        publisher=load('release_entry_public_reports','incremental-release-publish.py')
        publisher.verify_public_selection(self.transport,manifest,run['id'],run['run_attempt'],
            result[0].get('consumerReports'),upload_outputs=provenance.consumer_outputs(result[5],'public'))
        return result[:4]

    def run(self, run_id, attempt, source_sha, current_run=None):
        check(type(run_id) is int and run_id > 0 and type(attempt) is int and attempt > 0, "Exact release run and attempt required")
        run = self.transport.get("/actions/runs/" + str(run_id) + "/attempts/" + str(attempt))
        repository = self.transport.get("")
        check(run["id"] == run_id and run["run_attempt"] == attempt, "Release attempt differs")
        check(run["repository"]["id"] == repository["id"] == run["head_repository"]["id"], "Release repository differs or is a fork")
        check(run["path"] == WORKFLOW and run["head_sha"] == source_sha, "Release workflow or source differs")
        check(run["event"] in ["push", "workflow_dispatch"] and run["head_branch"] == repository["default_branch"], "Release proof must originate on main")
        live = current_run == {"runId": run_id, "attempt": attempt} and run["status"] == "in_progress"
        check(live or run["status"] == "completed" and run["conclusion"] == "success", "Release proof run has not succeeded")
        source = self.read_source(source_sha)
        route = "/branches/" + urllib.parse.quote(repository["default_branch"], safe="")
        policy_sha = self.transport.get(route)["commit"]["sha"]
        trusted = self.read_source(policy_sha)
        if source['policyHash'] != trusted['policyHash']:
            comparison=self.transport.get('/compare/'+source['sha']+'...'+policy_sha)
            check(comparison.get('status') in ['ahead','identical'] and comparison.get('base_commit',{}).get('sha')==source['sha']
                  and comparison.get('merge_base_commit',{}).get('sha')==source['sha'], 'Frozen release source is not proven trusted main history')
            # The original main run/source and its seal remain authoritative;
            # a later policy edit cannot invalidate an owned frozen transaction.
        jobs = execution.jobs_for(self.transport, run)
        return run, repository, source, jobs, route, policy_sha

    def verify(self, manifest, archive_directory, phase="complete", current_run=None):
        check(phase in ["prepublish", "postpublish", "complete"], "Unknown release verification phase")
        check(current_run is None or phase != "complete", "Completed verification cannot borrow a running execution")
        check(manifest.get("chain") == "release" and manifest.get("profile") == "release"
              and manifest.get("namespace") == "farm-release-v1", "CI bytes cannot enter release verification")
        check(manifest.get("publicationScope") == "npm" and all(row.get("kind") == "npm" for row in manifest.get("packages", [])), "New incremental publication is npm-only")
        chain.exact(manifest.get("sourceSHA"), 40)
        run, repository, source, jobs, route, policy_sha = self.run(manifest.get("producerRunId"), manifest.get("producerAttempt"), manifest["sourceSHA"], current_run)
        check(source["tree"] == manifest.get("sourceTree"), "Release source tree differs")
        # First-build path is independently recomputed. A later incremental
        # baseline must be verified by the durable published-ledger adapter;
        # accepting an uploaded baseline JSON here would be circular trust.
        baseline=published_baseline(self.transport,manifest['baseline'],self.read_source) if manifest.get('baseline') is not None and manifest.get('registrySnapshot') is None else None
        snapshot=manifest.get('registrySnapshot')
        presence = version_presence_mode(manifest)
        plan = source_plan(source,baseline,snapshot)
        if snapshot is not None and not presence:
            selector=load('verify_release_registry','release-registry-selection.py')
            registry = selector.Registry()
            selector.verify_retained(source,snapshot,lambda row: registry.lookup(row, verify_archive=True))
        check(plan["planHash"] == manifest.get("planHash") and plan["graphHash"] == manifest.get("graphHash"), "Release plan or graph differs from committed source")
        nodes, issues = chain.repository_graph(source["files"])
        check(not issues, "Release graph contains unresolved inputs")
        family = {key: row for key, row in chain.independent_family(source["files"], nodes).items() if row["kind"] == "npm"}
        declared=manifest.get('family')
        check(isinstance(declared,list) and len(declared)==len(family),'Manifest must bind the complete source family')
        declared_map={row['kind']+':'+row['name']:row for row in declared}
        check(set(declared_map)==set(family) and all(all(row.get(field)==family[key].get(field)
              for field in ['kind','name','version','manifestPath','owner','platform']) for key,row in declared_map.items()),'Manifest source family differs')
        check(manifest.get('rebuild')==plan['build'] and manifest.get('versionRequired')==plan['versionRequired']
              and manifest.get('publish')==plan['publish'],'Manifest rebuild/version/publication scope differs')
        check(not plan['publicationBlocked'],'Release plan has unresolved version or retirement requirements')
        rows = manifest.get("packages")
        check(isinstance(rows, list) and (presence or rows), "Complete release package inventory required")
        records = {row["kind"] + ":" + row["name"]: row for row in rows}
        if presence:
            failures = manifest.get('buildFailures')
            check(isinstance(failures, dict) and all(isinstance(value, str) and re.fullmatch(r'[a-z0-9][a-z0-9._:-]{0,119}', value) for value in failures.values()), 'Version-presence build failure inventory invalid')
            check(len(records) == len(rows) and not set(records) & set(failures)
                  and set(records) | set(failures) == set(plan['publish']), 'Release publication inventory is missing, duplicated or extra')
            check(set(plan['buildBlocked']) <= set(failures), 'Blocked build cannot claim a sealed source archive')
            for field in ['registryRetained', 'registryUnknown']:
                if field in manifest:
                    check(manifest[field] == plan[field], 'Manifest registry partition differs')
        else:
            check(len(records) == len(rows) and set(records) == set(plan['publish']), 'Release publication inventory is missing, duplicated or extra')
        archive_failures = []
        for identity, row in records.items():
            check(row["version"] == family[identity]["version"], "Release version differs from actual manifest")
            try:
                archive_valid = self.verify_archive(archive_directory, row) is True
            except (ValueError, OSError):
                if not presence or phase == 'prepublish':
                    raise
                archive_valid = False
            if not archive_valid:
                archive_failures.append(identity)
        receipt,artifact,origin,producer=self.receipt('frozen',run,source)
        check(origin['run_attempt']==manifest['producerAttempt'],'Frozen seal original attempt differs')
        manifest_hash = chain.digest(manifest)
        check(receipt.get("kind") == "release-seal" and receipt.get("profile") == "release"
              and receipt.get("namespace") == "farm-release-v1", "Release seal profile or namespace differs")
        check(receipt.get("manifestHash") == manifest_hash and receipt.get("producerJobId") == producer["id"]
              and receipt.get("sourceTree") == source["tree"], "Independent release seal binding differs")
        inputs = evidence.package_input_hashes(source)
        check(receipt.get("packageInputs") == inputs, "Release input closure differs")
        registry_failures = {}
        if presence and phase != 'prepublish':
            # Prepublication verifies source, plan and seal without a registry
            # batch. The writer validates each package freshly at its own write
            # boundary, so a slow retained version cannot starve independent work.
            # Final proof still rechecks the complete metadata observation.
            selector = load('verify_release_presence', 'release-registry-selection.py')
            registry_failures = selector.verify_retained(source, snapshot, selector.Registry().lookup, isolate=True)
        # Candidate installation is completed before the first registry write.
        # This gate must not require public installation of unpublished versions.
        if not archive_integrity_mode(manifest):
            for platform in ["ubuntu-24.04", "macos-latest", "windows-latest"]:
                evidence.successful_job(jobs, "Independent release candidate consumer (" + platform + ")")
        if phase == "prepublish":
            check(self.transport.get(route)["commit"]["sha"] == policy_sha, "Trusted release policy moved during verification")
            return {"manifestHash": manifest_hash, "sourceSHA": source["sha"], "sourceTree": source["tree"],
                    "sourceAndSealVerified": True, "archiveFailures": archive_failures,
                    **({"registryFailures": registry_failures, "buildFailures": manifest["buildFailures"]} if presence else {}),
                    "prepublicationVerified": not archive_failures and not registry_failures and (not presence or not manifest["buildFailures"] and not plan["registryUnknown"]), "verified": False, "publicationExecuted": False}
        if not presence:
            check(not archive_failures, 'Complete release proof requires every original archive')
        latest=self.transport.get('/actions/runs/'+str(run['id']))
        proof,artifact,_,consumer=self.receipt('public',latest,source,manifest)
        if archive_integrity_mode(manifest):
            check(proof.get('kind') == ('release-version-presence-v1' if presence else 'release-byte-integrity-v1'), 'Registry proof kind differs')
        check(proof.get("manifestHash") == manifest_hash and proof.get("producerJobId") == consumer["id"]
              and proof.get("sourceTree") == source["tree"] and proof.get("planHash") == plan["planHash"], "Public acceptance binding differs")
        if presence:
            check(not registry_failures, 'Retained registry verification failed')
            dispositions = verify_presence_dispositions(proof, manifest, plan, family, records)
            check(not any(dispositions[key]['originKind'] == 'source' for key in archive_failures), 'Complete release proof requires every new source archive')
            verify_current_presence(dispositions, family)
        else:
            check(proof.get('packages') == {key: row['archiveSha256'] for key, row in records.items()}, 'Public acceptance package coverage differs')
        check(public_integrity_verified(proof, manifest),
              "Actual registry, ABI and tag acceptance missing")
        check(self.transport.get(route)["commit"]["sha"] == policy_sha, "Trusted release policy moved during verification")
        return {"manifestHash": manifest_hash, "sourceSHA": source["sha"], "sourceTree": source["tree"],
                "producerRunId": run["id"], "producerAttempt": run["run_attempt"], "verified": True,
                **({"dispositions": dispositions, "packages": proof["packages"], "kind": proof["kind"]} if presence else {}),
                "publicationExecuted": False}


def actions_transport(environment=os.environ):
    check(environment.get("GITHUB_ACTIONS") == "true" and environment.get("GITHUB_REF") == "refs/heads/main"
          and environment.get("GITHUB_EVENT_NAME") in ["push", "workflow_dispatch"], "Live release reads require main Actions")
    repository = environment.get("GITHUB_REPOSITORY", "")
    check(environment.get("GITHUB_WORKFLOW_REF", "") == repository + "/" + WORKFLOW + "@refs/heads/main", "Unexpected executing release workflow")
    return evidence.transport_module.GitHubReadOnly(repository, environment.get("GH_TOKEN", ""), environment=environment, allow_archive=True)


def require_idle_publication(transport):
    """Early build admission only; never grants publication or recovery rights."""
    reader = load('build_admission_state', 'incremental-release-baseline.py')
    revision, saved = reader.ReleaseBaselineReader(transport, None).state()
    check('active' in saved and 'baseline' in saved, 'Invalid publication state')
    active = saved['active']
    if active is not None:
        state = load('build_admission_manifest', 'incremental-publication-state.py')
        check(isinstance(active, dict), 'Invalid active publication')
        manifest, owner = active.get('manifest', {}), active.get('owner', {})
        check(state.frozen_manifest(manifest) == active.get('transaction'), 'Active publication manifest differs')
        run, attempt = owner.get('runId'), owner.get('attempt')
        check(type(run) is int and run > 0 and type(attempt) is int and attempt > 0
              and owner.get('sourceSHA') == manifest.get('sourceSHA')
              and owner.get('producerRunId') == manifest.get('producerRunId')
              and active.get('phase') in ['uploading', 'public-acceptance', 'release-finalization'],
              'Invalid active publication owner')
        raise ValueError('publication-recovery-required: original run=' + str(run) + ' attempt=' + str(attempt)
                         + '; new release builds stopped; no lock or registry changes')
    return saved


def native_matrices(result):
    primary = {'linux-x64-gnu', 'darwin-arm64', 'win32-x64-msvc'}
    extra = {'linux-x64-musl', 'darwin-x64', 'win32-ia32-msvc', 'win32-arm64-msvc', 'linux-arm64-musl', 'linux-arm64-gnu'}
    by_platform = {}
    for cell in result.get('nativeBuildCells', []) + result.get('preparationNativeCells', []):
        check(cell['platform'] in primary | extra, 'Unsupported native release platform')
        by_platform.setdefault(cell['platform'], set()).add(cell['owner'])
    return {name: {'include': [{'abi': abi, 'native_owners': json.dumps(sorted(owners), separators=(',', ':'))}
        for abi, owners in sorted(by_platform.items()) if abi in platforms]}
        for name, platforms in [('native_primary', primary), ('native_extra', extra)]}


def main():
    parser = argparse.ArgumentParser()
    commands = parser.add_subparsers(dest="command", required=True)
    plan = commands.add_parser("plan"); plan.add_argument("--source", required=True); plan.add_argument("--output", required=True)
    mode = plan.add_mutually_exclusive_group()
    mode.add_argument('--full-source', action='store_true', help='Build every source npm package without reading historical release state')
    mode.add_argument('--version-presence', action='store_true', help='Build only exact npm versions proven missing, without historical release state')
    verify = commands.add_parser("verify"); verify.add_argument("--manifest", required=True); verify.add_argument("--archives", required=True)
    args = parser.parse_args()
    if args.command == "plan":
        source = evidence.source_reader(args.source)
        check(chain.impact.git("rev-parse", "HEAD").decode("ascii").strip() == source["sha"], "Checkout and release plan source differ")
        if args.version_presence:
            transport = actions_transport()
            selector = load('plan_release_version_presence', 'release-registry-selection.py')
            snapshot = selector.collect(source, 'npm', selector.Registry().lookup)
            history = load('plan_release_submission_history', 'release-version-history.py')
            current = {'runId': int(os.environ['GITHUB_RUN_ID']), 'attempt': int(os.environ['GITHUB_RUN_ATTEMPT'])}
            check(current['runId'] > 0 and 0 < current['attempt'] <= 20, 'Exact current planning attempt required')
            snapshot = history.guard(source, snapshot, transport, current=current)
            result = source_plan(source, registry_snapshot=snapshot)
        elif args.full_source:
            # A new source release is independent of historical transactions.
            # Registry absence is not an admission gate; each package is checked
            # at its exact-byte publication boundary and failures are recorded.
            actions_transport()
            result = source_plan(source)
        else:
            state = require_idle_publication(actions_transport())
            selector=load('plan_release_registry','release-registry-selection.py')
            registry=selector.Registry()
            snapshot=selector.collect(source,'npm',lambda row: registry.lookup(row,verify_archive=True),schema_version=1)
            snapshot['stateBaseline']=state['baseline']
            result = source_plan(source,registry_snapshot=snapshot)
        Path(args.output).write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
        if os.environ.get('GITHUB_OUTPUT'):
            records={row['kind']+':'+row['name']:row for row in result['fullFamily']}
            current_family=json.loads(source['files']['docs/releases/v2-candidate.json'])
            native_owners={row['owner'] for row in current_family['npm']['payloads']}
            owners=result['preparationNativeOwners']
            with Path(os.environ['GITHUB_OUTPUT']).open('a',encoding='utf-8') as stream:
                stream.write('native_required='+str(bool(owners)).lower()+'\npublish_required='+str(bool(result['publish'])).lower()
                             +'\ntransaction_required='+str(result.get('selectionPolicy')=='registry-exact-version-v2' or bool(result['publish'])).lower()
                             +'\nselection_policy='+result.get('selectionPolicy','source')
                             +'\nbuild_required='+str(bool(result['build'])).lower()+'\nregistry_unknown='+str(bool(result.get('registryUnknown'))).lower()+'\ngraph_hash='+result['graphHash']+'\nplan_hash='+result['planHash']+'\nnative_owners='+json.dumps(owners,separators=(',',':'))+'\n'
                             +'native_cells='+json.dumps(result.get('nativeBuildCells',[]),separators=(',',':'))+'\n'
                             +'preparation_native_cells='+json.dumps(result.get('preparationNativeCells',[]),separators=(',',':'))+'\n')
                for name, matrix in native_matrices(result).items():
                    stream.write(name+'_matrix='+json.dumps(matrix,separators=(',',':'))+'\n'
                                 +name+'_required='+str(bool(matrix['include'])).lower()+'\n')
        print(json.dumps({"chain": "release", "buildCount": len(result["build"]), "publicationExecuted": False}))
    else:
        manifest = json.loads(Path(args.manifest).read_text(encoding="utf-8"))
        expected = [("producerRunId", "EXPECTED_PRODUCER_RUN"), ("producerAttempt", "EXPECTED_PRODUCER_ATTEMPT"), ("sourceSHA", "EXPECTED_PRODUCER_SOURCE")]
        for field, key in expected:
            value = os.environ.get(key)
            check(value is not None and str(manifest.get(field)) == value, "Downloaded manifest differs from explicitly selected producer")
        print(json.dumps(ReleaseVerifier(actions_transport()).verify(manifest, args.archives)))


if __name__ == "__main__":
    main()
