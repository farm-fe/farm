"""Record actual release build/public-consumer outcomes in a fixed main workflow."""
# cspell:words argparse importlib npmjs urllib
import argparse
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import urllib.parse
import zipfile

spec = importlib.util.spec_from_file_location("release_record_entry", Path(__file__).with_name("incremental-release-entry.py"))
entry = importlib.util.module_from_spec(spec); spec.loader.exec_module(entry)
check = entry.check
selector = entry.load('record_version_selector', 'release-registry-selection.py')
best_effort = entry.load('record_best_effort', 'release-npm-best-effort.py')
history = entry.load('record_artifact_decoder', 'release-version-history.py')


def current_job(transport, expected):
    run_id, attempt = int(os.environ["GITHUB_RUN_ID"]), int(os.environ["GITHUB_RUN_ATTEMPT"])
    run = transport.get("/actions/runs/" + str(run_id) + "/attempts/" + str(attempt))
    check(run["id"] == run_id and run["run_attempt"] == attempt and run["head_sha"] == os.environ["GITHUB_SHA"]
          and run["path"] == entry.WORKFLOW and run["status"] == "in_progress", "Actual release recording identity differs")
    rows = [row for row in entry.execution.jobs_for(transport, run) if row["name"] == expected and row["status"] == "in_progress"]
    check(len(rows) == 1, "Exact running release producer job required")
    return rows[0]


def build_record(manifest, source, job, candidate):
    check(manifest["sourceSHA"] == source["sha"] and manifest["sourceTree"] == source["tree"], "Build source differs")
    receipt = json.loads((Path(candidate) / "receipt.json").read_text(encoding="utf-8"))
    check(receipt["status"] == "complete-seal" and receipt["testedCommit"] == source["sha"] and receipt["testedTree"] == source["tree"], "Independent candidate seal incomplete")
    for row in manifest["packages"]:
        entry.strict_archive(Path(candidate) / "archives", row)
    return {"kind": "release-seal", "profile": "release", "namespace": "farm-release-v1",
            "manifestHash": entry.chain.digest(manifest), "producerJobId": job["id"], "sourceTree": source["tree"],
            "packageInputs": entry.evidence.package_input_hashes(source)}


def check_archives(manifest, directory, source_sha, *, publisher_report=None):
    """Validate frozen bytes only; no consumer, scratch or test result is emitted."""
    check(manifest.get('sourceSHA') == source_sha, 'Archive source differs')
    check(manifest.get('chain') == 'release' and manifest.get('namespace') == 'farm-release-v1', 'Archive release namespace differs')
    rows = manifest.get('packages')
    check(isinstance(rows, list) and (rows or entry.version_presence_mode(manifest)), 'Archive inventory missing')
    check(len({row['kind'] + ':' + row['name'] for row in rows}) == len(rows), 'Duplicate archive identity')
    if publisher_report is not None:
        check(entry.version_presence_mode(manifest), 'Only version-presence dispositions can select source archives')
        _, outcomes = validate_publisher_report(manifest, publisher_report, publisher_report.get('runId'), publisher_report.get('attempt'))
        rows = [row for row in rows if outcomes['npm:' + row['name']]['originKind'] == 'source']
    for row in rows:
        entry.strict_archive(Path(directory) / 'archives', row)


def public_record(manifest, reports, job, registry):
    check(not entry.archive_integrity_mode(manifest), "Legacy public reports require the original consumer policy")
    expected = {"ubuntu-24.04", "macos-latest", "windows-latest"}
    check(set(reports) == expected, "All three public consumer reports required")
    for report in reports.values():
        check(report.get("status") == "passed" and report.get("publicRegistry") is True
              and report.get("workspacePatches") is False and report.get("cleanupConfirmed") is not False,
              "Public registry consumer or cleanup failed")
        check(report.get('kind') != 'release-byte-integrity-v1', 'Legacy consumer reports cannot use byte-only proof')
        required={"npm-install", "esm-cjs", "types", "docs-build"} | (set() if manifest.get("publicationScope") == "npm" else {"sdk"})
        check(required.issubset(report.get("stages", [])), "Actual public consumer stages missing")
    return registry_record(manifest, job, registry, integrity=False)


def registry_record(manifest, job, registry, *, integrity=True):
    """Registry bytes and stable tag read-back, without installation or tests."""
    check(not entry.version_presence_mode(manifest), "Version-presence proof requires authenticated publisher dispositions")
    with registry.budget(1500):
        return _registry_record(manifest, job, registry, integrity=integrity)


def _registry_record(manifest, job, registry, *, integrity):
    check(entry.archive_integrity_mode(manifest) == integrity, "Frozen manifest and registry proof policy differ")
    records = {row["kind"] + ":" + row["name"]: row for row in manifest["packages"]}
    actual = registry.verify(records)
    check(actual == {key: row["archiveSha256"] for key, row in records.items()}, "Independent public registry bytes differ")
    for row in records.values():
        if row["kind"] == "npm":
            latest = registry.json("https://registry.npmjs.org/" + urllib.parse.quote(row["name"], safe="") + "/latest")
            check(latest.get("name") == row["name"], "Public latest package identity differs")
            def version(value):
                check(isinstance(value, str) and entry.re.fullmatch(r"\d+\.\d+\.\d+", value), "Exact stable latest version required")
                return tuple(map(int, value.split(".")))
            check(version(latest.get("version")) >= version(row["version"]), "Public latest tag is behind selected stable release")
    return {"manifestHash": entry.chain.digest(manifest), "producerJobId": job["id"], "sourceTree": manifest["sourceTree"],
            "planHash": manifest["planHash"], "packages": actual, "registryInstall": not integrity, "nativeABI": not integrity,
            "latestTargets": True, **({'kind': 'release-byte-integrity-v1', 'registryBytesVerified': True, 'testsExecuted': False} if integrity else {})}


def validate_publisher_report(manifest, report, run_id, attempt):
    """A complete report is evidence only after its artifact origin is verified."""
    check(entry.version_presence_mode(manifest), 'Version-presence policy required')
    best_effort.new_report(manifest)  # Validate full family and intended build partition.
    check(not manifest.get('buildFailures') and not any(
        row['status'] == 'unknown' for row in manifest['registrySnapshot']['packages'].values()),
        'Complete public proof cannot hide unknown versions or build failures')
    check(isinstance(report, dict) and report.get('schemaVersion') == 2
          and report.get('kind') == 'npm-package-publication-results' and report.get('stage') == 'publisher'
          and report.get('selectionPolicy') == manifest['selectionPolicy'], 'Publisher result policy differs')
    check(report.get('complete') is True and report.get('success') is True
          and report.get('baselineAdvanced') is False and report.get('releaseCreatedByThisStage') is False
          and not report.get('fatal') and not report.get('errors'), 'Publisher result is incomplete or failed')
    check(type(run_id) is int and run_id > 0 and type(attempt) is int and attempt > 0
          and type(report.get('runId')) is int and report['runId'] == run_id == manifest['producerRunId']
          and type(report.get('attempt')) is int and report['attempt'] == attempt
          and type(manifest.get('producerAttempt')) is int and 0 < manifest['producerAttempt'] <= attempt,
          'Publisher result execution differs')
    for key in ['producerRunId', 'producerAttempt']:
        check(type(report.get(key)) is int, 'Publisher original producer identity invalid')
    for key in ['sourceSHA', 'sourceTree', 'producerRunId', 'producerAttempt']:
        check(report.get(key) == manifest[key], 'Publisher result source or original producer differs')
    check(report.get('manifestHash') == entry.chain.digest(manifest), 'Publisher result frozen manifest differs')
    family = {'npm:' + row['name']: row for row in manifest['family']}
    rows = report.get('packages')
    check(isinstance(rows, list) and len(rows) == len(family)
          and all(isinstance(row, dict) for row in rows), 'Publisher result family missing')
    outcomes = {'npm:' + row.get('name', ''): row for row in rows}
    check(len(outcomes) == len(rows) and set(outcomes) == set(family), 'Publisher result family differs')
    for key, row in outcomes.items():
        check(row.get('version') == family[key]['version'] and row.get('status') == 'success'
              and row.get('errors') == [] and row.get('registryMetadataVerified') is True
              and row.get('originKind') in ['source', 'registry', 'registry-race']
              and row.get('registryBytesVerified') is (row['originKind'] == 'source')
              and row.get('latestVerified') is True and row.get('outcomeUncertain') is False
              and row.get('latestOutcomeUncertain') is False and type(row.get('uploadAttempted')) is bool,
              'Publisher package is incomplete or failed')
        selector.validate_observed(family[key], {**row, 'status': 'published'}, 2)
        check(row.get('evidenceKind') == ('registry-archive' if row['originKind'] == 'source' else 'registry-metadata'),
              'Publisher package observation kind differs')
        if row['originKind'] != 'source':
            check('archiveSha256' not in row and 'archiveBytes' not in row, 'Metadata-only publisher result claims archive verification')
    expected_counts = {'total': len(rows), 'success': len(rows), 'failed': 0,
                       'published': sum(row['uploadAttempted'] for row in rows),
                       'retained': sum(not row['uploadAttempted'] for row in rows)}
    check(isinstance(report.get('counts'), dict) and all(type(value) is int and value >= 0 for value in report['counts'].values())
          and report['counts'] == expected_counts, 'Publisher result counts differ')
    return family, outcomes


def authenticated_publisher_report(transport, manifest, local_file, environment=None):
    """Bind local JSON to the exact immutable upload, never a filename alone."""
    env = os.environ if environment is None else environment
    artifact_id = env.get('EXPECTED_PUBLISHER_RESULT_ARTIFACT_ID', '')
    expected_digest = env.get('EXPECTED_PUBLISHER_RESULT_ARTIFACT_DIGEST', '')
    check(isinstance(artifact_id, str) and entry.re.fullmatch(r'[1-9][0-9]{0,19}', artifact_id)
          and isinstance(expected_digest, str) and entry.re.fullmatch('[a-f0-9]{64}', expected_digest),
          'Trusted publisher artifact output required')
    run_id, attempt = int(env['GITHUB_RUN_ID']), int(env['GITHUB_RUN_ATTEMPT'])
    repository = transport.get('')
    check(type(repository.get('id')) is int and repository['id'] > 0 and repository.get('default_branch') == 'main',
          'Publisher repository differs')
    run = transport.get('/actions/runs/' + str(run_id) + '/attempts/' + str(attempt))
    check(run.get('id') == run_id and run.get('run_attempt') == attempt
          and run.get('head_sha') == manifest['sourceSHA'] == env['GITHUB_SHA']
          and run.get('repository', {}).get('id') == run.get('head_repository', {}).get('id') == repository['id']
          and run.get('path') == entry.WORKFLOW and run.get('head_branch') == 'main'
          and run.get('event') == 'push' and run.get('status') == 'in_progress', 'Publisher run origin differs')
    jobs = entry.execution.jobs_for(transport, run)
    rows = [row for row in jobs if row.get('name') == entry.PUBLISHER]
    check(len(rows) == 1, 'Publisher job missing or ambiguous')
    job = rows[0]
    check(type(job.get('id')) is int and job['id'] > 0 and job.get('run_id') == run_id
          and job.get('run_attempt') == attempt and job.get('head_sha') == manifest['sourceSHA']
          and job.get('status') == 'completed' and job.get('conclusion') == 'success', 'Publisher job did not succeed in this attempt')
    artifact = transport.get('/actions/artifacts/' + artifact_id)
    check(type(artifact.get('id')) is int and artifact['id'] == int(artifact_id)
          and artifact.get('name') == 'npm-publication-results-' + str(run_id) + '-' + str(attempt)
          and artifact.get('expired') is False and artifact.get('digest') == 'sha256:' + expected_digest
          and type(artifact.get('size_in_bytes')) is int and 0 < artifact['size_in_bytes'] <= history.MAX_BYTES,
          'Publisher artifact identity differs')
    origin = artifact.get('workflow_run', {})
    check(origin.get('id') == run_id and origin.get('head_sha') == manifest['sourceSHA']
          and origin.get('repository_id') == origin.get('head_repository_id') == repository['id'],
          'Publisher artifact source differs')
    entry.execution.upload_binding(job, history.UPLOAD_STEP, artifact,
        {'artifact_id': artifact_id, 'artifact_digest': expected_digest})
    body = transport.archive(artifact['id'])
    report = history.decode_report(body, artifact)
    # decode_report has already checked exactly one bounded regular member.
    with zipfile.ZipFile(io.BytesIO(body)) as archive:
        authenticated_bytes = archive.read('results.json')
    file = Path(local_file)
    check(file.is_file() and not file.is_symlink() and file.stat().st_size == len(authenticated_bytes)
          and file.read_bytes() == authenticated_bytes, 'Local publisher results differ from authenticated upload')
    validate_publisher_report(manifest, report, run_id, attempt)
    return report, {'artifactId': artifact['id'], 'artifactDigest': artifact['digest'],
                    'runId': run_id, 'attempt': attempt, 'publisherJobId': job['id']}


def presence_registry_record(manifest, job, registry, report):
    """Fresh whole-family observations preserve registry versus source origins."""
    family, outcomes = validate_publisher_report(manifest, report, report.get('runId'), report.get('attempt'))
    archives = {'npm:' + row['name']: row for row in manifest['packages']}
    dispositions = {}
    # Only source-origin publication archives are read again. Retained/race
    # observations prove exact versions and dependency metadata, not tar bytes.
    check(all(key in archives for key, row in outcomes.items() if row['originKind'] == 'source'), 'Source disposition has no frozen archive')
    byte_limit = sum(archives[key]['archiveBytes'] for key, row in outcomes.items() if row['originKind'] == 'source')
    check(0 <= byte_limit <= len(family) * 100 * 1024 ** 2, 'Public registry inventory byte bound differs')
    registry.total_byte_limit = max(1, byte_limit)
    registry_lookup = selector.Registry(registry)
    metadata_fields = ['name', 'version', 'integrity', 'metadata', 'tarballURL', 'evidenceKind']
    with registry.budget(1500):
        for key, row in sorted(family.items()):
            outcome = outcomes[key]
            origin, attempted = outcome.get('originKind'), outcome['uploadAttempted']
            planned = manifest['registrySnapshot']['packages'][key]
            if planned['status'] == 'published':
                check(origin == 'registry' and attempted is False, 'Retained registry origin differs')
            else:
                check(planned['status'] == 'missing' and key in archives, 'Missing source candidate absent')
                check((origin == 'source' and attempted is True) or (origin == 'registry-race' and attempted is False),
                      'New source or registry race origin differs')
            source_origin = origin == 'source'
            actual = registry_lookup.lookup(row, verify_archive=source_origin)
            selector.validate_observed(row, actual, 2)
            check(actual['status'] == 'published', 'Public exact version disappeared')
            fields = metadata_fields + (['archiveSha256', 'archiveBytes'] if source_origin else [])
            check(all(actual.get(field) == outcome.get(field) for field in fields), 'Public registry changed since publisher confirmation')
            if planned['status'] == 'published':
                check(all(actual[field] == planned[field] for field in ['integrity', 'metadata', 'tarballURL']),
                      'Retained registry observation changed')
            if source_origin:
                check(actual['archiveSha256'] == archives[key]['archiveSha256']
                      and actual['archiveBytes'] == archives[key]['archiveBytes'], 'New source archive origin differs')
            dispositions[key] = {field: actual[field] for field in fields}
            dispositions[key].update(originKind=origin, uploadAttempted=attempted, latestVerified=True,
                                     registryBytesVerified=source_origin)
        metadata = {row['name']: row['metadata'] for row in dispositions.values()}
        # Validate actual public dependency/peer ranges, not fresh source metadata.
        for peers_only in [False, True]:
            dependencies = best_effort.required_dependencies(list(family.values()), metadata, peers_only=peers_only)
            check(set(dependencies) == {row['name'] for row in family.values()}
                  and all(names.issubset(metadata) for names in dependencies.values()), 'Public dependency closure incomplete')
        for row in family.values():
            latest = registry.json('https://registry.npmjs.org/' + urllib.parse.quote(row['name'], safe='') + '/latest')
            check(latest.get('name') == row['name'], 'Public latest package identity differs')
            def version(value):
                check(isinstance(value, str) and entry.re.fullmatch(r'\d+\.\d+\.\d+', value), 'Exact stable latest version required')
                return tuple(map(int, value.split('.')))
            check(version(latest.get('version')) >= version(row['version']), 'Public latest tag is behind selected stable release')
    proof = {'kind': 'release-version-presence-v1', 'manifestHash': entry.chain.digest(manifest),
             'producerJobId': job['id'], 'sourceTree': manifest['sourceTree'], 'planHash': manifest['planHash'],
             'packages': {key: row.get('archiveSha256') for key, row in dispositions.items()}, 'dispositions': dispositions,
             'latestTargets': True, 'registryMetadataVerified': True, 'registryInstall': False, 'nativeABI': False, 'testsExecuted': False}
    entry.verify_presence_dispositions(proof, manifest,
        {'registryUnknown': {}, 'registrySnapshot': manifest['registrySnapshot']}, family, archives)
    return proof


def main():
    parser = argparse.ArgumentParser(); parser.add_argument("stage", choices=["build", "public", "registry"])
    parser.add_argument("--candidate", required=True); parser.add_argument("--output", required=True)
    parser.add_argument("--reports"); parser.add_argument("--publisher-results"); args = parser.parse_args()
    transport = entry.actions_transport()
    manifest = json.loads((Path(args.candidate) / "manifest.json").read_text(encoding="utf-8"))
    if args.stage == "registry":
        check(entry.archive_integrity_mode(manifest), "Registry-only receipt requires the signed archive-integrity policy")
        job = current_job(transport, entry.CONSUMER)
        if entry.version_presence_mode(manifest):
            check(args.publisher_results is not None, 'Authenticated publisher result artifact required')
            report, binding = authenticated_publisher_report(transport, manifest, args.publisher_results)
            check_archives(manifest, args.candidate, os.environ['GITHUB_SHA'], publisher_report=report)
            record = presence_registry_record(manifest, job, selector.ExactRegistryReadOnly(), report)
            record['publisherResults'] = binding
        else:
            check_archives(manifest, args.candidate, os.environ['GITHUB_SHA'])
            record = registry_record(manifest, job, entry.evidence.registry_module.RegistryReadOnly())
    elif args.stage == "build":
        record = build_record(manifest, entry.evidence.source_reader(os.environ["GITHUB_SHA"]), current_job(transport, entry.PRODUCER), args.candidate)
    else:
        check(args.reports is not None, "Public consumer reports required")
        reports = {platform: json.loads((Path(args.reports) / ("release-public-" + platform) / "result.json").read_text(encoding="utf-8"))
                   for platform in ["ubuntu-24.04", "macos-latest", "windows-latest"]}
        record = public_record(manifest, reports, current_job(transport, entry.CONSUMER), entry.evidence.registry_module.RegistryReadOnly())
    output = Path(args.output); output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"stage": args.stage, "publicationExecuted": False}))


if __name__ == "__main__":
    main()
