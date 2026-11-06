"""Verify functional acceptance of the exact release tree, without CI build bytes."""
# cspell:words importlib IFMT ISREG redispatch
import argparse
import hashlib
import os
import time
import importlib.util
import io
import json
from pathlib import Path
import stat
import zipfile

spec = importlib.util.spec_from_file_location("functional_release_entry", Path(__file__).with_name("incremental-release-entry.py"))
entry = importlib.util.module_from_spec(spec); spec.loader.exec_module(entry)
check = entry.check
retry_spec = importlib.util.spec_from_file_location('release_retry_safety', Path(__file__).with_name('release-retry-safety.py'))
retry = importlib.util.module_from_spec(retry_spec); retry_spec.loader.exec_module(retry)
PLATFORMS = ["ubuntu-24.04", "macos-latest", "windows-latest"]
FUNCTIONAL = ["Release source contracts", "Check declared native inputs (not ABI execution)", "Type Check",
              *[name + " (" + platform + ")" for name in ["Examples Test", "Typescript Test"] for platform in PLATFORMS],
              "Docs / windows-latest", "Docs / ubuntu-24.04", "Record verified CI execution baseline"]
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


def main_binding_proof(transport, run, source):
    consumer = entry.load('functional_signed_consumer', 'ci-provenance-consumer.py')
    verifier = entry.load('functional_pinned_cli', 'ci-pinned-attestation-cli.py')
    trust = json.loads(Path(__file__).with_name('ci-provenance-trust.json').read_text())
    check(isinstance(trust, dict) and set(trust) == {'schemaVersion', 'signerSHA'} and trust['schemaVersion'] == 1,
          'Reviewed signer trust configuration differs')
    return consumer.load_verified_run(transport, run, source, trust['signerSHA'], verify=verifier.verify_on_host)


def verify_functional(transport, source, bindings, current_run=None, *, proof_provider=None):
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
        latest = transport.get('/actions/runs/' + str(run['id']))
        check(all(latest.get(key) == run.get(key) for key in ['id','run_attempt','path','head_sha','event','status','conclusion']),
              'Functional latest attempt differs; prior success cannot authorize publication')
        tested = transport.get("/git/commits/" + binding["testedSHA"])
        check(tested["tree"]["sha"] == source["tree"], "Actually tested functional tree differs from release source")
        check(run['event'] == 'push' and run['head_branch'] == repository['default_branch']
              and run['head_sha'] == binding['testedSHA'] == source['sha'], 'Functional default-branch checkout differs')
        jobs = entry.execution.jobs_for(transport, run)
        for name in required:
            job = entry.evidence.successful_job(jobs, name)
            check(job.get('run_id') == run['id'] and job.get('head_sha') == run['head_sha']
                  and job.get('run_attempt') == run['run_attempt'], 'Functional current attempt job identity differs')
        expected, signed = (main_binding_proof if proof_provider is None else proof_provider)(transport, run, source)
        check('planner' in signed and signed['planner']['producer'] == planner, 'Signed functional planner role missing')
        consumer = entry.load('functional_attested_binding', 'ci-attested-bindings.py')
        body, artifact, planner_run, planner_job = consumer.bound_artifact(transport, run, signed['planner'], expected)
        plan = exact_plan(body, artifact)
        check(plan.get("testedSHA") == binding["testedSHA"] and plan.get("testedTree") == source["tree"]
              and str(plan.get("runId")) == str(run["id"]) and str(plan.get("runAttempt")) == str(planner_run["run_attempt"])
              and plan.get("repository") == repository["full_name"], "Functional plan execution identity differs")
        check(plan.get("planHash") == entry.chain.digest({key: value for key, value in plan.items() if key != "planHash"}), "Functional plan hash differs")
        check(not plan.get('planningFallback') and not plan.get('unavailable'), 'Functional main source plan unavailable')
        source_plan = importlib.util.spec_from_file_location('functional_source_plan', Path(__file__).with_name('ci-source-plan.py'))
        verifier = importlib.util.module_from_spec(source_plan); source_plan.loader.exec_module(verifier)
        verifier.verify_main_plan(plan, 'e2e' if scope == 'ci' else 'unit')
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



def discover_main(transport, source, current_run):
    repository = transport.get(''); result = {}
    for scope, file in [('ci', 'ci.yaml'), ('unit', 'rust-test.yaml')]:
        runs = transport.get('/actions/workflows/' + file + '/runs?per_page=20&event=push&head_sha=' + source['sha'])['workflow_runs']
        matches = [run for run in runs if run.get('head_sha') == source['sha'] and run.get('head_branch') == repository['default_branch']
                   and run.get('repository', {}).get('id') == repository['id'] == run.get('head_repository', {}).get('id')]
        check(matches, 'Exact main functional run is not ready; wait and redispatch without publication')
        run = max(matches, key=lambda row: row['id'])
        check(run.get('status') == 'completed' and run.get('conclusion') == 'success',
              'Exact main functional run is not ready; wait and redispatch without publication')
        result[scope] = {'runId': run['id'], 'attempt': run['run_attempt'], 'testedSHA': source['sha']}
    result.update({scope: dict(current_run) for scope in ['lint', 'spell']})
    return result


def original_attempt(transport, run_id, source_sha, current_attempt):
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
                workflow=entry.WORKFLOW, producer=entry.PRODUCER, freeze_step='Freeze independent release candidates',
                publisher='Publish exact independent release archives under durable lock', events=['push','workflow_dispatch'])
            print(json.dumps({'unfrozenRetryProof': proof}), flush=True)
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


PENDING_MESSAGE = 'Exact main functional run is not ready; wait and redispatch without publication'


def wait_for_main(transport, source, current, *, seconds, clock=time.monotonic, sleep=time.sleep):
    """A bounded wait preserves a recoverable pending state, never acceptance."""
    deadline = clock() + seconds
    while True:
        try:
            return discover_main(transport, source, current)
        except ValueError as error:
            if str(error) != PENDING_MESSAGE or clock() >= deadline:
                raise
            sleep(min(30, max(0, deadline - clock())))


def pending_record(source, current):
    return {'schemaVersion': 1, 'kind': 'release-functional-pending',
            'sourceSHA': source['sha'], 'sourceTree': source['tree'],
            'releaseRunId': current['runId'], 'releaseAttempt': current['attempt'],
            'reason': 'exact-main-evidence-not-ready', 'publicationAuthorized': False}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(); parser.add_argument('--source', required=True); parser.add_argument('--output', required=True); parser.add_argument('--wait', action='store_true')
    args = parser.parse_args(); transport = entry.actions_transport()
    check(args.source == os.environ['GITHUB_SHA'], 'Functional source must be the fixed release SHA')
    source = entry.evidence.source_reader(args.source)
    current = {'runId': int(os.environ['GITHUB_RUN_ID']), 'attempt': int(os.environ['GITHUB_RUN_ATTEMPT'])}
    supplied = os.environ.get('FARM_FUNCTIONAL_EVIDENCE', '')
    try:
        bindings = json.loads(supplied) if supplied else wait_for_main(transport, source, current, seconds=120 * 60 if args.wait else 0)
    except ValueError as error:
        if not supplied and str(error) == PENDING_MESSAGE:
            Path('release-functional-pending.json').write_text(json.dumps(pending_record(source, current), sort_keys=True) + '\n', encoding='utf-8')
        raise
    verify_functional(transport, source, bindings, current)
    Path(args.output).write_text(json.dumps(bindings, sort_keys=True) + '\n', encoding='utf-8')
