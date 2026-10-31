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
chain = evidence.chain_plan
check = evidence.check
WORKFLOW = ".github/workflows/incremental-release.yaml"
PRODUCER = "Build independent release candidates"
CONSUMER = "Verify independent release public consumers"


def source_plan(source, baseline=None):
    family = json.loads(source["files"]["docs/releases/v2-candidate.json"])
    abi=re.search(r'^pub const VERSION: &str = "([0-9]+\.[0-9]+\.[0-9]+)";',source['files']['crates/core/src/lib.rs'].decode('utf-8'),re.M)
    check(abi is not None and family['abi']==abi[1],'Candidate Farm ABI differs from actual source declaration')
    prior=baseline['source'] if baseline else None
    result=chain.plan_chain("release", {"sha": source["sha"], "tree": source["tree"]}, source["files"], family, [],
                            baseline=baseline['ledger'] if baseline else None,baseline_files=prior['files'] if prior else None,
                            anchor=baseline['anchor'] if baseline else None,source_modes=source["modes"],baseline_modes=prior['modes'] if prior else None)
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
    baseline=published_baseline(transport,manifest['baseline']) if manifest.get('baseline') is not None else None
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


class ReleaseVerifier:
    def __init__(self, transport, read_source=evidence.source_reader):
        self.transport, self.read_source = transport, read_source

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
        chain.exact(manifest.get("sourceSHA"), 40)
        run, repository, source, jobs, route, policy_sha = self.run(manifest.get("producerRunId"), manifest.get("producerAttempt"), manifest["sourceSHA"], current_run)
        check(source["tree"] == manifest.get("sourceTree"), "Release source tree differs")
        # First-build path is independently recomputed. A later incremental
        # baseline must be verified by the durable published-ledger adapter;
        # accepting an uploaded baseline JSON here would be circular trust.
        baseline=published_baseline(self.transport,manifest['baseline'],self.read_source) if manifest.get('baseline') is not None else None
        plan = source_plan(source,baseline)
        check(plan["planHash"] == manifest.get("planHash"), "Release plan differs from committed source")
        nodes, issues = chain.repository_graph(source["files"])
        check(not issues, "Release graph contains unresolved inputs")
        family = chain.independent_family(source["files"], nodes)
        declared=manifest.get('family')
        check(isinstance(declared,list) and len(declared)==len(family),'Manifest must bind the complete source family')
        declared_map={row['kind']+':'+row['name']:row for row in declared}
        check(set(declared_map)==set(family) and all(all(row.get(field)==family[key].get(field)
              for field in ['kind','name','version','manifestPath','owner','platform']) for key,row in declared_map.items()),'Manifest source family differs')
        check(manifest.get('rebuild')==plan['build'] and manifest.get('versionRequired')==plan['versionRequired']
              and manifest.get('publish')==plan['publish'],'Manifest rebuild/version/publication scope differs')
        check(not plan['publicationBlocked'],'Release plan has unresolved version or retirement requirements')
        rows = manifest.get("packages")
        check(isinstance(rows, list) and rows, "Complete release package inventory required")
        records = {row["kind"] + ":" + row["name"]: row for row in rows}
        check(len(records) == len(rows) and set(records) == set(plan["publish"]), "Release publication inventory is missing, duplicated or extra")
        for identity, row in records.items():
            check(row["version"] == family[identity]["version"], "Release version differs from actual manifest")
            strict_archive(archive_directory, row)
        producer = evidence.successful_job(jobs, PRODUCER)
        name = "farm-release-seal-" + source["sha"] + "-" + str(run["run_attempt"])
        receipt, artifact = execution.fetch_json_artifact(self.transport, run, name)
        execution.upload_binding(producer, "Upload independent release seal", artifact)
        manifest_hash = chain.digest(manifest)
        check(receipt.get("kind") == "release-seal" and receipt.get("profile") == "release"
              and receipt.get("namespace") == "farm-release-v1", "Release seal profile or namespace differs")
        check(receipt.get("manifestHash") == manifest_hash and receipt.get("producerJobId") == producer["id"]
              and receipt.get("sourceTree") == source["tree"], "Independent release seal binding differs")
        inputs = evidence.package_input_hashes(source)
        check(receipt.get("packageInputs") == inputs, "Release input closure differs")
        # Candidate installation is completed before the first registry write.
        # This gate must not require public installation of unpublished versions.
        for platform in ["ubuntu-24.04", "macos-latest", "windows-latest"]:
            evidence.successful_job(jobs, "Independent release candidate consumer (" + platform + ")")
        if phase == "prepublish":
            check(self.transport.get(route)["commit"]["sha"] == policy_sha, "Trusted release policy moved during verification")
            return {"manifestHash": manifest_hash, "sourceSHA": source["sha"], "sourceTree": source["tree"],
                    "prepublicationVerified": True, "verified": False, "publicationExecuted": False}
        consumer = evidence.successful_job(jobs, CONSUMER)
        proof, artifact = execution.fetch_json_artifact(self.transport, run, "farm-release-public-acceptance-" + source["sha"] + "-" + str(run["run_attempt"]))
        execution.upload_binding(consumer, "Upload independent release public acceptance", artifact)
        check(proof.get("manifestHash") == manifest_hash and proof.get("producerJobId") == consumer["id"]
              and proof.get("sourceTree") == source["tree"] and proof.get("planHash") == plan["planHash"], "Public acceptance binding differs")
        check(proof.get("packages") == {key: row["archiveSha256"] for key, row in records.items()}, "Public acceptance package coverage differs")
        check(proof.get("registryInstall") is True and proof.get("nativeABI") is True and proof.get("latestTargets") is True,
              "Actual registry, ABI and tag acceptance missing")
        check(self.transport.get(route)["commit"]["sha"] == policy_sha, "Trusted release policy moved during verification")
        return {"manifestHash": manifest_hash, "sourceSHA": source["sha"], "sourceTree": source["tree"],
                "producerRunId": run["id"], "producerAttempt": run["run_attempt"], "verified": True,
                "publicationExecuted": False}


def actions_transport(environment=os.environ):
    check(environment.get("GITHUB_ACTIONS") == "true" and environment.get("GITHUB_REF") == "refs/heads/main"
          and environment.get("GITHUB_EVENT_NAME") in ["push", "workflow_dispatch"], "Live release reads require main Actions")
    repository = environment.get("GITHUB_REPOSITORY", "")
    check(environment.get("GITHUB_WORKFLOW_REF", "") == repository + "/" + WORKFLOW + "@refs/heads/main", "Unexpected executing release workflow")
    return evidence.transport_module.GitHubReadOnly(repository, environment.get("GH_TOKEN", ""), environment=environment, allow_archive=True)


def main():
    parser = argparse.ArgumentParser()
    commands = parser.add_subparsers(dest="command", required=True)
    plan = commands.add_parser("plan"); plan.add_argument("--source", required=True); plan.add_argument("--output", required=True)
    verify = commands.add_parser("verify"); verify.add_argument("--manifest", required=True); verify.add_argument("--archives", required=True)
    args = parser.parse_args()
    if args.command == "plan":
        source = evidence.source_reader(args.source)
        check(chain.impact.git("rev-parse", "HEAD").decode("ascii").strip() == source["sha"], "Checkout and release plan source differ")
        baseline=published_baseline(actions_transport()) if os.environ.get('GITHUB_REF')=='refs/heads/main' and os.environ.get('GITHUB_ACTIONS')=='true' else None
        result = source_plan(source,baseline)
        Path(args.output).write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
        if os.environ.get('GITHUB_OUTPUT'):
            records={row['kind']+':'+row['name']:row for row in result['fullFamily']}
            current_family=json.loads(source['files']['docs/releases/v2-candidate.json'])
            native_owners={row['owner'] for row in current_family['npm']['payloads']}
            owners=sorted({records[key].get('owner') or records[key]['name'] for key in result['build']
                           if records[key].get('owner') or records[key]['name'] in native_owners})
            with Path(os.environ['GITHUB_OUTPUT']).open('a',encoding='utf-8') as stream:
                stream.write('native_required='+str(bool(owners)).lower()+'\npublish_required='+str(bool(result['publish'])).lower()
                             +'\nplan_hash='+result['planHash']+'\nnative_owners='+json.dumps(owners,separators=(',',':'))+'\n')
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
