"""Verify functional acceptance of the exact release tree, without CI build bytes."""
# cspell:words importlib IFMT ISREG redispatch
import argparse
import ast
import re
import hashlib
import math
import os
import time
import urllib.parse
import importlib.util
import io
import json
from pathlib import Path
import stat
import zipfile

spec = importlib.util.spec_from_file_location("functional_release_entry", Path(__file__).with_name("incremental-release-entry.py"))
entry = importlib.util.module_from_spec(spec); spec.loader.exec_module(entry)
check = entry.check
PLATFORMS = ["ubuntu-24.04", "macos-latest", "windows-latest"]
FUNCTIONAL = ["Release source contracts", "Check declared native inputs (not ABI execution)", "Type Check & Changesets",
              *[name + " (" + platform + ")" for name in ["Examples Test", "Typescript Test", "Commerce Performance"] for platform in PLATFORMS],
              "Docs / windows-latest", "Docs / ubuntu-24.04", "Aggregate informational performance report"]
# The other four original functional roles (seal + three candidate consumers)
# are verified from the independent release run by ReleaseVerifier.prepublish.


def exact_plan(body, artifact, member="plan.json"):
    check(0 < len(body) <= 4 * 1024 * 1024 and len(body) == artifact["size_in_bytes"]
          and artifact["digest"] == "sha256:" + hashlib.sha256(body).hexdigest(), "Functional plan archive digest differs")
    with zipfile.ZipFile(io.BytesIO(body)) as archive:
        rows = archive.infolist()
        check(len(rows) == 1 and rows[0].filename == member and 0 < rows[0].file_size <= 4 * 1024 * 1024
              and stat.S_IFMT(rows[0].external_attr >> 16) in [0, stat.S_IFREG], "Functional plan archive members differ")
        return json.loads(archive.read(rows[0]).decode("utf-8"))


def verify_functional(transport, source, bindings, current_run=None):
    check(isinstance(bindings, dict) and set(bindings) in [{"ci", "unit", "lint", "spell"}, {"ci", "unit", "lint", "spell", "title"}],
          "Exact CI, Unit and quality workflow acceptance runs required")
    repository = transport.get("")
    accepted_head = None
    accepted_base = None
    for scope, path, planner, required in [
        ("ci", ".github/workflows/ci.yaml", "Incremental CI plan (shadow only)", FUNCTIONAL),
        ("unit", ".github/workflows/rust-test.yaml", "Incremental Unit CI plan (shadow only)", ["Check", *["Test - " + platform for platform in PLATFORMS]])]:
        binding = bindings[scope]
        check(type(binding.get("runId")) is int and binding["runId"] > 0 and type(binding.get("attempt")) is int and binding["attempt"] > 0, "Exact acceptance run/attempt required")
        entry.chain.exact(binding.get("testedSHA"), 40)
        run = transport.get("/actions/runs/" + str(binding["runId"]) + "/attempts/" + str(binding["attempt"]))
        check(run["id"] == binding["runId"] and run["run_attempt"] == binding["attempt"] and run["path"] == path
              and run["status"] == "completed" and run["conclusion"] == "success", "Functional workflow did not succeed")
        check(run["repository"]["id"] == repository["id"] == run["head_repository"]["id"]
              and run["event"] in ["pull_request", "workflow_dispatch", "push"], "Functional run repository/event differs")
        tested = transport.get("/git/commits/" + binding["testedSHA"])
        check(tested["tree"]["sha"] == source["tree"], "Actually tested functional tree differs from release source")
        jobs = entry.execution.jobs_for(transport, run)
        attempts = {}
        for name in required:
            entry.execution.actual_producer(transport, run, name, jobs, attempts)
        planner_run, planner_job = entry.execution.actual_producer(transport, run, planner, jobs, attempts)
        name = "impact-plan-" + binding["testedSHA"] + "-" + str(planner_run["run_attempt"])
        response = transport.get("/actions/runs/" + str(run["id"]) + "/artifacts?per_page=2&name=" + name)
        rows = response["artifacts"]
        check(response.get("total_count") == 1 and len(rows) == 1 and rows[0]["name"] == name and rows[0]["expired"] is False, "Exact functional source artifact missing")
        artifact = transport.get("/actions/artifacts/" + str(rows[0]["id"]))
        check(artifact["workflow_run"]["id"] == run["id"] and artifact["name"] == name and artifact["expired"] is False
              and 0 < artifact["size_in_bytes"] <= 4 * 1024 * 1024, "Functional artifact producer differs")
        entry.execution.upload_binding(planner_job, 'Upload fixed source impact plan', artifact)
        plan = exact_plan(transport.archive(artifact["id"]), artifact)
        check(plan.get("testedSHA") == binding["testedSHA"] and plan.get("testedTree") == source["tree"]
              and str(plan.get("runId")) == str(run["id"]) and str(plan.get("runAttempt")) == str(planner_run["run_attempt"])
              and plan.get("repository") == repository["full_name"], "Functional plan execution identity differs")
        check(plan.get("planHash") == entry.chain.digest({key: value for key, value in plan.items() if key != "planHash"}), "Functional plan hash differs")
        if run["event"] == "pull_request":
            parents = {row["sha"] for row in tested.get("parents", [])}
            check(plan.get("headSHA") == run["head_sha"] and {plan.get("headSHA"), plan.get("baseSHA")} == parents,
                  "Functional checkout is not its actual PR test merge")
            check(any(row.get("head", {}).get("sha") == plan["headSHA"] and row.get("base", {}).get("sha") == plan["baseSHA"]
                      for row in run.get("pull_requests", [])), "Functional PR source association differs")
        else:
            check(run["event"] == "push" and run["head_branch"] == repository["default_branch"]
                  and run["head_sha"] == binding["testedSHA"] == source['sha'], "Functional default-branch checkout differs")
            check(not plan.get('planningFallback') and not plan.get('unavailable'), 'Functional main source plan unavailable')
        if scope == 'ci':
            verify_performance(transport, run, jobs, source, plan)
        if scope == "ci":
            accepted_head, accepted_base = run["head_sha"], plan.get("baseSHA")
    for scope, path in [("lint", ".github/workflows/lint.yaml"), ("spell", ".github/workflows/code-spell-check.yaml"), ("title", ".github/workflows/lint-pr-title.yml")]:
        if scope not in bindings:
            check(scope == 'title' and accepted_head == source['sha'], 'PR title acceptance missing')
            continue
        binding = bindings[scope]
        check(type(binding.get("runId")) is int and binding["runId"] > 0 and type(binding.get("attempt")) is int and binding["attempt"] > 0, "Exact quality workflow identity required")
        run = transport.get("/actions/runs/" + str(binding["runId"]) + "/attempts/" + str(binding["attempt"]))
        own_quality = scope in ['lint', 'spell'] and current_run is not None and current_run['runId'] == run['id'] and 0 < run['run_attempt'] <= current_run['attempt'] and run['path'] == entry.WORKFLOW
        if own_quality:
            check(run['repository']['id'] == repository['id'] == run['head_repository']['id']
                  and run['event'] in ['push', 'workflow_dispatch'] and run['head_branch'] == repository['default_branch']
                  and run['head_sha'] == source['sha'] and run['status'] in ['in_progress', 'completed'], 'Main quality run differs')
            quality = entry.evidence.successful_job(entry.execution.jobs_for(transport, run), 'Release quality checks')
            steps = ['Check release Rust formatting', 'Check release Rust lint', 'Check release TS lint'] if scope == 'lint' else ['Check release spelling']
            for name in steps:
                check(len([step for step in quality.get('steps', []) if step.get('name') == name and step.get('status') == 'completed' and step.get('conclusion') == 'success']) == 1,
                      'Actual main quality step missing or failed')
            continue
        check(run["id"] == binding["runId"] and run["run_attempt"] == binding["attempt"] and run["path"] == path
              and run["repository"]["id"] == repository["id"] == run["head_repository"]["id"]
              and run["status"] == "completed" and run["conclusion"] == "success" and run["head_sha"] == accepted_head,
              "Required quality workflow differs, is stale or did not succeed")
        if run["event"] == "pull_request":
            check(any(row.get("head", {}).get("sha") == accepted_head and row.get("base", {}).get("sha") == accepted_base
                      for row in run.get("pull_requests", [])), "Quality workflow PR merge inputs differ")
        else:
            check(run["event"] in ["push", "workflow_dispatch"] and run["head_branch"] == repository["default_branch"], "Quality workflow event differs")
    return {"sourceTree": source["tree"], "functionalAcceptanceVerified": True, "buildBytesReused": False}


def fetch_member(transport, run, name, member):
    response = transport.get('/actions/runs/' + str(run['id']) + '/artifacts?per_page=2&name=' + urllib.parse.quote(name, safe=''))
    rows = response['artifacts']
    check(response.get('total_count') == 1 and len(rows) == 1, 'Exact performance artifact missing')
    artifact = transport.get('/actions/artifacts/' + str(rows[0]['id']))
    check(artifact['name'] == name and artifact['workflow_run']['id'] == run['id'] and artifact['expired'] is False,
          'Performance artifact producer differs')
    return exact_plan(transport.archive(artifact['id']), artifact, member), artifact


def verify_measurement_source(transport, run, source, summary, row, proof, observations=None):
    check(isinstance(proof, dict) and proof.get('profile') == 'ci' and proof.get('workflowPath') == '.github/workflows/ci.yaml', 'Measurement chain differs')
    systems = {'linux': ('linux-x64-gnu', 'ubuntu-24.04', 'Primary Linux'), 'darwin': ('darwin-arm64', 'macos-latest', 'Primary macOS'), 'win32': ('win32-x64-msvc', 'windows-latest', 'Primary Windows')}
    abi, runner, platform = systems[row['metadata']['os']]
    check(proof.get('abi') == abi and proof.get('runId') == run['id'] and proof.get('producerSHA') == source['sha'] and proof.get('testedTree') == source['tree'], 'Measurement source differs')
    for key in ['runAttempt', 'nativeAttempt']:
        check(type(proof.get(key)) is int and 0 < proof[key] <= run['run_attempt'], 'Invalid measurement attempt')
    check(type(run.get('run_attempt')) is int and 0 < run['run_attempt'] <= 32, 'Measurement attempts exceed bound')
    observations = {} if observations is None else observations
    def observed(attempt):
        if attempt in observations: return observations[attempt]
        original = transport.get('/actions/runs/' + str(run['id']) + '/attempts/' + str(attempt))
        check(original['id'] == run['id'] and original['run_attempt'] == attempt and original['head_sha'] == run['head_sha'] and original['path'] == run['path'] and original['event'] == run['event'] and original['repository']['id'] == run['repository']['id'], 'Original measurement run differs')
        rows = []
        for page in range(1, 21):
            batch = transport.get('/actions/runs/' + str(run['id']) + '/attempts/' + str(attempt) + '/jobs?per_page=100&page=' + str(page))['jobs']
            rows.extend(batch)
            if len(batch) < 100: break
        else: raise ValueError('Measurement jobs exceed bound')
        observations[attempt] = (original, rows)
        return original, rows
    def actual(attempt, name, job):
        check(type(job.get('id')) is int and job['id'] > 0 and job.get('run_id') == run['id'], 'Measurement job identity differs')
        check(job.get('started_at') and job.get('completed_at') and job.get('status') == 'completed' and job.get('conclusion') == 'success', 'Latest actual measurement producer did not succeed')
        if attempt == 1: return True
        _, previous = observed(attempt - 1)
        matching = [item for item in previous if item['name'] == name]
        check(len(matching) <= 1, 'Ambiguous previous measurement producer')
        if not matching: return True
        prior = matching[0]
        if (prior.get('started_at'), prior.get('completed_at')) == (job.get('started_at'), job.get('completed_at')):
            check(prior.get('id') == job.get('id') and prior.get('status') == 'completed' and prior.get('conclusion') == 'success', 'Unproven carried measurement producer')
            return False
        check(not prior.get('completed_at') or job['started_at'] >= prior['completed_at'], 'Producer execution times overlap across attempts')
        return True
    def producer(attempt, name, job_id):
        _, rows = observed(attempt)
        job = entry.evidence.successful_job(rows, name)
        check(job['id'] == job_id and actual(attempt, name, job), 'Carried job is not a new producer')
        for later in range(attempt + 1, run['run_attempt'] + 1):
            _, later_jobs = observed(later)
            matching = [item for item in later_jobs if item['name'] == name]
            check(len(matching) <= 1, 'Ambiguous later measurement producer')
            check(not any(actual(later, name, item) for item in matching), 'A newer actual producer cannot be hidden')
        return job
    def member(artifact_id, digest, name, job, filename):
        artifact = transport.get('/actions/artifacts/' + str(artifact_id))
        check(artifact['id'] == artifact_id and artifact['digest'] == digest and artifact['name'] == name and not artifact['expired'] and artifact['workflow_run']['id'] == run['id'] and artifact['workflow_run']['repository_id'] == run['repository']['id'] and artifact['workflow_run']['head_sha'] == run['head_sha'], 'Measurement artifact differs')
        check(job['started_at'] <= artifact['created_at'] <= job['completed_at'], 'Measurement upload outside producer')
        body = transport.archive(artifact_id)
        check(0 < len(body) <= 16*1024*1024 and len(body) == artifact['size_in_bytes'] and digest == 'sha256:' + hashlib.sha256(body).hexdigest(), 'Measurement archive bytes differ')
        with zipfile.ZipFile(io.BytesIO(body)) as archive:
            entries = archive.infolist(); names = [item.filename for item in entries]
            check(len(entries) <= 500 and len(set(names)) == len(names), 'Ambiguous measurement archive')
            check(all(not item.flag_bits & 1 and stat.S_IFMT(item.external_attr >> 16) in [0, stat.S_IFREG, stat.S_IFDIR] and not item.filename.startswith('/') and '\\' not in item.filename and ':' not in item.filename and all(part not in ['.', '..', ''] for part in item.filename.rstrip('/').split('/')) for item in entries), 'Unsafe measurement archive')
            item = archive.getinfo(filename); check(0 < item.file_size <= 2*1024*1024, 'Measurement member exceeds bound')
            content = archive.read(item)
            return json.loads(content.decode('utf-8')), hashlib.sha256(content).hexdigest()
    job = producer(proof['runAttempt'], 'Commerce Performance (' + runner + ')', proof['jobId'])
    raw, digest = member(proof['artifactId'], proof['artifactDigest'], 'performance-' + source['sha'] + '-' + abi + '-attempt-' + str(proof['runAttempt']), job, 'summary.json')
    check(digest == proof['rawSummarySha256'] and raw.get('status') == 'complete' and raw.get('protocol') == summary['protocol'], 'Raw measurement differs')
    for key, value in row['metadata'].items(): check(raw.get('metadata', {}).get(key) == value, 'Compacted measurement metadata differs')
    check(raw['metadata']['artifactProducerSHA'] == source['sha'] and raw['metadata']['harnessSha256'] == proof['harnessSha256'], 'Measurement input differs')
    check(raw.get('cells') == row['cells'] and raw.get('comparison', {}).get('mode') == row['comparison']['mode'] and raw.get('comparison', {}).get('baseCells', []) == row['comparison']['baseCells'], 'Compacted measurement cells differ')
    native_kind = proof.get('nativeKind', 'build')
    check(native_kind in ['build','materialization'], 'Unknown native source kind')
    reused = native_kind == 'materialization'
    check(not reused or abi == 'linux-x64-gnu', 'Unsupported native reuse platform')
    native = producer(proof['nativeAttempt'], 'Type Check & Changesets' if reused else platform + ' / native / Build and Upload User Artifacts - ' + abi, proof['nativeJobId'])
    name = 'ci-consumer-native-proof-' + str(run['id']) + '-' + str(proof['nativeAttempt']) if reused else 'native-shadow-' + source['sha'] + '-' + abi + '-' + str(proof['nativeAttempt'])
    products, _ = member(proof['nativeArtifactId'], proof['nativeArtifactDigest'], name, native, 'consumer.json' if reused else 'products.json')
    check(products.get('profile') == 'ci' and products.get('abi') == abi and str(products.get('runId')) == str(run['id']), 'Measured native producer differs')
    if reused:
        claim = raw.get('metadata',{}).get('nativeReuse')
        check(isinstance(claim,dict) and claim.get('chain') == 'ci' and claim.get('consumerSHA') == source['sha'] and claim.get('acceptanceInherited') is False, 'Measured reuse claim differs')
        check(products.get('kind') == 'ci-consumer-materialization-not-a-new-build' and products.get('chain') == 'ci'
              and products.get('consumerSHA') == source['sha'] and products.get('consumerTree') == source['tree']
              and str(products.get('attempt')) == str(proof['nativeAttempt']) and products.get('acceptanceInherited') is False
              and products.get('originContext',{}).get('profile') == 'ci'
              and products.get('originBuildSHA') == claim.get('originBuildSHA')
              and str(products.get('originArtifactId')) == str(claim.get('originArtifactId')), 'Measured original materialization differs')
    else:
        check(raw.get('metadata',{}).get('nativeReuse') is None and products.get('originBuildSHA') == source['sha'] and str(products.get('runAttempt')) == str(proof['nativeAttempt']) and products.get('repository') == run['repository']['full_name'], 'Measured native build producer differs')
    for key in ['jobId', 'artifactId', 'nativeJobId', 'nativeArtifactId']:
        check(type(proof.get(key)) is int and proof[key] > 0, 'Invalid measurement producer ID')
    records = products.get('records', []); owners = {item['owner']: item for item in records}
    check(len(owners) == len(records) and raw.get('native', {}).get('artifacts'), 'Measured native inventory absent')
    policy_bytes = source['files']['scripts/performance-cohorts.mjs']
    check(isinstance(policy_bytes, bytes), 'Measured cohort policy must be source bytes')
    policy_text = policy_bytes.decode('utf-8', errors='strict')
    declaration = re.search(r'export const nativePlugins = (\[[^;]+\]);', policy_text)
    check(declaration is not None, 'Measured cohort policy missing')
    plugins = ast.literal_eval(declaration[1])
    check(isinstance(plugins, list) and plugins and len(set(plugins)) == len(plugins) and all(isinstance(item, str) and re.fullmatch(r'[a-z-]+', item) for item in plugins), 'Measured cohort policy differs')
    required = {'@farmfe/core', *['@farmfe/plugin-' + item for item in plugins]}
    check(set(raw['native']['artifacts']) == required, 'Incomplete measured native cohort')
    for owner, actual in raw['native']['artifacts'].items():
        check(owner in owners and owners[owner]['sha256'] == actual['sha256'] and owners[owner]['bytes'] == actual['bytes'], 'Measured native bytes differ')


def verify_performance(transport, run, jobs, source, plan):
    aggregation, producer = entry.execution.actual_producer(transport, run, 'Aggregate informational performance report', jobs)
    summary, artifact = fetch_member(transport, run, 'performance-pr-summary-' + str(aggregation['run_attempt']), 'summary.json')
    check(producer['started_at'] <= artifact['created_at'] <= producer['completed_at'], 'Performance artifact outside successful attempt')
    check(summary.get('schemaVersion') in [2, 3] and summary.get('protocol') == 'commerce-business-2048-farm-supervised-v2', 'Performance protocol differs')
    check(summary.get('runId') == run['id'] and summary.get('runAttempt') == aggregation['run_attempt']
          and summary.get('repositoryId') == run['repository']['id'] and summary.get('event') == run['event']
          and summary.get('headSHA') == run['head_sha'] and summary.get('baseSHA') == plan.get('baseSHA'), 'Performance source identity differs')
    check(summary.get('prNumber') is None if run['event'] == 'push' else type(summary.get('prNumber')) is int, 'Performance event association differs')
    reports = summary.get('reports', [])
    check(len(reports) == 3 and {(row.get('metadata', {}).get('os'), row.get('metadata', {}).get('arch')) for row in reports}
          == {('linux', 'x64'), ('darwin', 'arm64'), ('win32', 'x64')}, 'Three performance platforms required')
    if summary['schemaVersion'] == 3:
        proofs = summary.get('measurementSources')
        check(isinstance(proofs, list) and len(proofs) == len(reports), 'Measurement sources missing')
        observations = {}
        for row, proof in zip(reports, proofs): verify_measurement_source(transport, run, source, summary, row, proof, observations)
    else:
        check(run['run_attempt'] == 1, 'Legacy summary cannot authorize mixed attempts')
    for row in reports:
        meta = row['metadata']
        check(row.get('status') == 'complete' and meta.get('revision') == source['sha'] and meta.get('baseSHA') == summary['baseSHA'], 'Incomplete or stale performance report')
        for key in ['fixtureSha256', 'lockSha256']:
            entry.chain.exact(meta.get(key), 64)
        cells = row.get('cells', [])
        check(len(cells) == 2 and {cell.get('cacheMode') for cell in cells} == {'cold', 'warm'}, 'Cold and warm performance samples required')
        for cell in cells:
            check(type(cell.get('samples')) is int and cell['samples'] >= 3 and type(cell.get('validSamples')) is int
                  and cell['validSamples'] == cell['samples'], 'Incomplete actual performance samples')
            check(all(type(cell.get(key)) in [int, float] and math.isfinite(cell[key]) and cell[key] >= 0 for key in ['medianMs', 'iqrMs'])
                  and cell['medianMs'] > 0, 'Invalid actual performance values')
    return summary


def discover_main(transport, source, current_run):
    repository = transport.get(''); result = {}
    for scope, file in [('ci', 'ci.yaml'), ('unit', 'rust-test.yaml')]:
        runs = transport.get('/actions/workflows/' + file + '/runs?per_page=20&event=push&head_sha=' + source['sha'])['workflow_runs']
        matches = [run for run in runs if run.get('head_sha') == source['sha'] and run.get('head_branch') == repository['default_branch']
                   and run.get('status') == 'completed' and run.get('conclusion') == 'success'
                   and run.get('repository', {}).get('id') == repository['id'] == run.get('head_repository', {}).get('id')]
        check(matches, 'Exact main functional run is not ready; wait and redispatch without publication')
        run = max(matches, key=lambda row: row['id'])
        result[scope] = {'runId': run['id'], 'attempt': run['run_attempt'], 'testedSHA': source['sha']}
    result.update({scope: dict(current_run) for scope in ['lint', 'spell']})
    return result


def original_attempt(transport, run_id, source_sha, current_attempt):
    prefix = 'farm-release-candidates-' + source_sha + '-'; matches = []
    for page in range(1, 4):
        rows = transport.get('/actions/runs/' + str(run_id) + '/artifacts?per_page=100&page=' + str(page))['artifacts']
        matches.extend(row for row in rows if row['name'].startswith(prefix) and row['name'][len(prefix):].isdigit())
        if len(rows) < 100:
            break
    if not matches:
        return current_attempt
    check(len(matches) == 1, 'Multiple frozen candidates cannot form one release transaction')
    artifact = matches[0]; attempt = int(artifact['name'][len(prefix):])
    check(artifact.get('expired') is False and 0 < attempt <= current_attempt and artifact['workflow_run']['id'] == run_id,
          'Original frozen release archive expired or differs; do not rebuild recovery')
    run = transport.get('/actions/runs/' + str(run_id) + '/attempts/' + str(attempt))
    check(run['id'] == run_id and run['run_attempt'] == attempt and run['path'] == entry.WORKFLOW
          and run['head_sha'] == source_sha and run['head_branch'] == 'main', 'Original release producer source differs')
    producer = entry.evidence.successful_job(entry.execution.jobs_for(transport, run), entry.PRODUCER)
    seal_name = 'farm-release-seal-' + source_sha + '-' + str(attempt)
    response = transport.get('/actions/runs/' + str(run_id) + '/artifacts?per_page=2&name=' + seal_name)
    seals = response['artifacts']
    check(response.get('total_count') == 1 and len(seals) == 1 and seals[0]['name'] == seal_name, 'Original release seal is missing or ambiguous')
    seal = transport.get('/actions/artifacts/' + str(seals[0]['id']))
    check(seal['name'] == seal_name and seal['expired'] is False and seal['workflow_run']['id'] == run_id, 'Original release seal producer differs')
    entry.execution.upload_binding(producer, 'Upload independent release seal', seal)
    check(producer['started_at'] <= artifact['created_at'] <= producer['completed_at'], 'Candidate artifact outside original producer')
    return attempt


if __name__ == '__main__':
    parser = argparse.ArgumentParser(); parser.add_argument('--source', required=True); parser.add_argument('--output', required=True); parser.add_argument('--wait', action='store_true')
    args = parser.parse_args(); transport = entry.actions_transport()
    check(args.source == os.environ['GITHUB_SHA'], 'Functional source must be the fixed release SHA')
    source = entry.evidence.source_reader(args.source)
    current = {'runId': int(os.environ['GITHUB_RUN_ID']), 'attempt': int(os.environ['GITHUB_RUN_ATTEMPT'])}
    supplied = os.environ.get('FARM_FUNCTIONAL_EVIDENCE', '')
    deadline = time.monotonic() + (120 * 60 if args.wait else 0)
    while True:
        try:
            bindings = json.loads(supplied) if supplied else discover_main(transport, source, current)
            break
        except ValueError as error:
            if str(error) != 'Exact main functional run is not ready; wait and redispatch without publication' or time.monotonic() >= deadline:
                raise
            time.sleep(30)
    verify_functional(transport, source, bindings, current)
    Path(args.output).write_text(json.dumps(bindings, sort_keys=True) + '\n', encoding='utf-8')
