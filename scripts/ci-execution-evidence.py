"""Verify E2E acceptance ledgers for a narrow docs-only path; never reuse build bytes."""
# cspell:words importlib IFMT IFREG
import hashlib
import importlib.util
import json
from pathlib import Path

spec = importlib.util.spec_from_file_location("execution_evidence", Path(__file__).with_name("incremental-chain-evidence.py"))
evidence = importlib.util.module_from_spec(spec)
spec.loader.exec_module(evidence)
chain = evidence.chain_plan
runtime = evidence.load("execution_suite_policy", "incremental-chain-runtime.py")
check = evidence.check
PLATFORMS = {"linux-x64-gnu": "ubuntu-24.04", "darwin-arm64": "macos-latest", "win32-x64-msvc": "windows-latest"}
PRODUCER = "Aggregate informational performance report"
UPLOAD = "Upload trusted CI execution baseline"


def leaf_file(file, mode):
    return file.startswith("docs/experiences/") and file.endswith(".md") and mode == "100644"


def acceptance_inputs(source):
    rows = chain.snapshot_records(source["files"], source["modes"])
    return chain.digest({file: row for file, row in rows.items() if not leaf_file(file, row["mode"])})


def jobs_for(transport, run):
    jobs = []
    for page in range(1, 4):
        rows = transport.get("/actions/runs/" + str(run["id"]) + "/attempts/" + str(run["run_attempt"])
                             + "/jobs?per_page=100&page=" + str(page))["jobs"]
        jobs.extend(rows)
        if len(rows) < 100:
            return jobs
    raise ValueError("Execution job inventory exceeded bound")


def fetch_json_artifact(transport, run, name, member="ledger.json"):
    response = transport.get("/actions/runs/" + str(run["id"]) + "/artifacts?per_page=2&name="
                             + evidence.transport_module.urllib.parse.quote(name, safe=""))
    rows = [row for row in response["artifacts"] if row["name"] == name and row["expired"] is False]
    check(len(rows) == 1 and response.get("total_count", len(rows)) == 1, "Exact execution artifact missing or ambiguous")
    artifact = transport.get("/actions/artifacts/" + str(rows[0]["id"]))
    check(artifact["workflow_run"]["id"] == run["id"] and artifact["expired"] is False and artifact["name"] == name, "Execution artifact origin differs")
    check(0 < artifact["size_in_bytes"] <= evidence.MAX_LEDGER_BYTES, "Execution artifact exceeds bound")
    body = transport.archive(artifact["id"])
    # The existing ZIP verifier accepts exactly one canonical regular JSON file.
    # Unit/suite receipts use another fixed member, never an artifact-supplied path.
    if member == "ledger.json":
        data = evidence.ledger_member(body, artifact)
    else:
        import io
        import stat
        import zipfile
        check(member in ["ts.json", "browser.json", "result.json", "selection.json"], "Unexpected execution receipt member")
        check(len(body) == artifact["size_in_bytes"] and artifact["digest"] == "sha256:" + hashlib.sha256(body).hexdigest(), "Execution artifact bytes differ")
        with zipfile.ZipFile(io.BytesIO(body)) as archive:
            entries = archive.infolist()
            check(len(entries) == 1 and entries[0].filename == member and 0 < entries[0].file_size <= evidence.MAX_LEDGER_BYTES,
                  "Execution receipt members differ")
            check(stat.S_IFMT(entries[0].external_attr >> 16) in [0, stat.S_IFREG], "Execution receipt is not a regular file")
            data = json.loads(archive.read(member).decode("utf-8"))
    return data, artifact


def upload_binding(job, step_name, artifact):
    steps = [step for step in job.get("steps", []) if step.get("name") == step_name]
    check(len(steps) == 1 and steps[0]["status"] == "completed" and steps[0]["conclusion"] == "success", "Actual producer upload did not succeed")
    check(steps[0]["started_at"] <= artifact["created_at"] <= steps[0]["completed_at"], "Artifact is outside its fixed upload step")


def actual_producer(transport, run, name, current_jobs=None, cache=None):
    """Newest real execution wins, including failures; carried jobs retain origin."""
    check(type(run['run_attempt']) is int and 0 < run['run_attempt'] <= 32, 'Rerun attempt exceeds bound')
    observations = {} if cache is None else cache
    for attempt in range(run['run_attempt'], 0, -1):
        if attempt in observations: continue
        execution = run if attempt == run['run_attempt'] and current_jobs is not None else transport.get('/actions/runs/' + str(run['id']) + '/attempts/' + str(attempt))
        check(all(execution.get(key) == run.get(key) for key in ['id', 'path', 'head_sha', 'event'])
              and execution.get('run_attempt') == attempt
              and execution.get('repository', {}).get('id') == run['repository']['id']
              and execution.get('head_repository', {}).get('id') == run['head_repository']['id'], 'Rerun source or workflow differs')
        observations[attempt] = (execution, current_jobs if attempt == run['run_attempt'] and current_jobs is not None else jobs_for(transport, execution))
    for attempt in range(run['run_attempt'], 0, -1):
        execution, jobs = observations[attempt]
        matches = [job for job in jobs if job['name'] == name or job['name'].endswith(' / ' + name)]
        check(len(matches) <= 1, 'Ambiguous suite producer')
        if not matches: continue
        job = matches[0]
        check(type(job.get('id')) is int and job['id'] > 0 and job.get('run_id') == run['id'], 'Suite producer job identity differs')
        check(job.get('started_at') and job.get('completed_at') and job.get('status') == 'completed' and job.get('conclusion') == 'success', 'Latest actual suite producer did not succeed')
        prior = [item for item in observations.get(attempt - 1, (None, []))[1] if item['name'] == name or item['name'].endswith(' / ' + name)]
        check(len(prior) <= 1, 'Ambiguous prior suite producer')
        if prior and (job['started_at'], job.get('completed_at')) == (prior[0].get('started_at'), prior[0].get('completed_at')):
            check(job.get('id') == prior[0].get('id') and prior[0].get('status') == 'completed' and prior[0].get('conclusion') == 'success', 'Unproven carried suite producer')
            continue
        check(not prior or not prior[0].get('completed_at') or job['started_at'] >= prior[0]['completed_at'], 'Suite producer times overlap')
        check(job.get('run_id') == run['id'] and job.get('status') == 'completed' and job.get('conclusion') == 'success', 'Latest actual suite producer did not succeed')
        return execution, job
    raise ValueError('Actual suite producer missing')


def suite_receipt(transport, run, source, kind, abi, current_jobs=None, cache=None):
    label, filename, job_label, step = ('ts', 'ts.json', 'Typescript Test', 'Preserve actual TypeScript discovery and execution counts') if kind == 'ts' else ('browser', 'browser.json', 'Examples Test', 'Preserve actual browser discovery and execution counts')
    prefix = {'linux-x64-gnu':'Primary Linux', 'darwin-arm64':'Primary macOS', 'win32-x64-msvc':'Primary Windows'}[abi]
    execution, job = actual_producer(transport, run, prefix + ' / ' + job_label + ' (' + PLATFORMS[abi] + ')', current_jobs, cache)
    name = 'farm-ci-suite-' + label + '-' + source['sha'] + '-' + abi + '-' + str(execution['run_attempt'])
    actual, meta = fetch_json_artifact(transport, execution, name, filename)
    upload_binding(job, step, meta)
    expected = {'repositoryId':run['repository']['id'], 'runId':run['id'], 'runAttempt':execution['run_attempt'], 'event':run['event']}
    check(actual.get('run') == expected and actual.get('sourceSHA') == source['sha'] and actual.get('sourceTree') == source['tree'], 'Original suite source or attempt differs')
    check(actual.get('chain') == 'ci' and actual.get('acceptanceInherited') is False
          and actual.get('kind') == kind and actual.get('abi') == abi, 'Original suite profile or cell differs')
    return actual, meta, {'runAttempt':execution['run_attempt'], 'jobId':job['id'], 'artifactId':meta['id'],
                          'artifactDigest':meta['digest'], 'profile':'ci', 'workflowPath':run['path']}


def verify_execution_ledger(transport, run, repository, ledger, artifact, jobs, source, trusted, required_jobs,
                            suite_discovery=runtime.discover_source_suites):
    check(run["repository"]["id"] == repository["id"] == run["head_repository"]["id"], "Execution baseline repository differs or is a fork")
    check(run["path"] == evidence.WORKFLOWS["ci"] and run["status"] == "completed" and run["conclusion"] == "success", "CI execution baseline did not complete")
    check(run["event"] in ["pull_request", "push", "workflow_dispatch"], "Unexpected CI baseline event")
    if run["event"] == "pull_request":
        prs = run.get("pull_requests", [])
        check(len(prs) == 1, "Exact PR source association missing")
        if source["sha"] != run["head_sha"]:
            check(prs[0]["head"]["sha"] == run["head_sha"] and {prs[0]["head"]["sha"], prs[0]["base"]["sha"]} == set(source["parents"]), "Actual CI checkout is not the verified merge")
    else:
        check(run["head_branch"] == repository["default_branch"] and source["sha"] == run["head_sha"], "Default-branch source differs")
    check(source["policyHash"] == trusted["policyHash"], "CI producer code is not the trusted current policy")
    check(ledger.get("schemaVersion") == 2 and ledger.get("kind") == "ci-execution-baseline-no-build-byte-authority", "Legacy or byte-authority ledger is not an execution baseline")
    check(ledger.get("chain") == "ci" and ledger.get("namespace") == "farm-ci-execution-v1", "Execution namespace differs")
    check(ledger.get("scope") == "e2e-only" and ledger.get("unitExecutionRequired") is True, "Execution baseline cannot omit Unit validation")
    check(ledger.get("sourceSHA") == source["sha"] and ledger.get("sourceTree") == source["tree"] and ledger.get("policyHash") == trusted["policyHash"], "Execution source or policy differs")
    check(ledger.get("sourceSnapshotHash") == chain.digest(chain.snapshot_records(source["files"], source["modes"])), "Execution source snapshot differs")
    check(ledger.get("acceptanceInputsHash") == acceptance_inputs(source), "Execution input closure differs")
    check(ledger.get("buildByteReuseAuthorized") is False and ledger.get("publicationAuthorized") is False, "Execution ledger claims unauthorized authority")
    check(ledger.get("producerJob") == PRODUCER and ledger.get("uploadStep") == UPLOAD, "Ledger cannot choose its producer")
    attempts = {}
    aggregation, producer = actual_producer(transport, run, PRODUCER, jobs, attempts)
    check(artifact.get("name") == "farm-ci-execution-baseline-" + str(run["id"]) + "-" + str(aggregation["run_attempt"])
          and artifact["workflow_run"]["id"] == run["id"] and artifact["expired"] is False, "Execution ledger artifact origin differs")
    expected_run = {"repositoryId": repository["id"], "runId": run["id"], "runAttempt": aggregation["run_attempt"], "event": run["event"]}
    check(ledger.get("run") == expected_run, "Execution attempt differs")
    check(sorted(ledger.get("requiredJobs", [])) == sorted(required_jobs), "Trusted full acceptance inventory differs")
    for name in required_jobs:
        _, job = actual_producer(transport, run, name, jobs, attempts)
        check(job["run_id"] == run["id"], "Acceptance job belongs to another run")
    upload_binding(producer, UPLOAD, artifact)
    cells = ledger.get("coverage", {}).get("cells", [])
    expected_cells = {kind + ":" + abi for kind in ["ts", "examples-browser"] for abi in PLATFORMS}
    check(len(cells) == 6 and {row["kind"] + ":" + row["abi"] for row in cells} == expected_cells, "Missing three-platform semantic coverage")
    discovered = suite_discovery(source, trusted)
    check(discovered["sourceIdentity"]["commit"] == source["sha"] and discovered["sourceIdentity"]["tree"] == source["tree"], "Independent suite source differs")
    for cell in cells:
        label, filename, job_label, step = ("ts", "ts.json", "Typescript Test", "Preserve actual TypeScript discovery and execution counts") if cell["kind"] == "ts" else ("browser", "browser.json", "Examples Test", "Preserve actual browser discovery and execution counts")
        if 'producerBinding' in cell:
            actual, meta, binding = suite_receipt(transport, run, source, cell['kind'], cell['abi'], jobs, attempts)
            check(binding == cell['producerBinding'], 'Suite original producer proof differs')
            check(cell['artifactName'] == meta['name'], 'Suite original artifact name differs')
        else:
            name = "farm-ci-suite-" + label + "-" + source["sha"] + "-" + cell["abi"] + "-" + str(aggregation["run_attempt"])
            actual, meta = fetch_json_artifact(transport, run, name, filename)
            _, job = actual_producer(transport, run, job_label + " (" + PLATFORMS[cell["abi"]] + ")", jobs, attempts)
            upload_binding(job, step, meta)
            check(actual["run"] == expected_run, 'Actual semantic attempt differs')
        check(actual == {key: value for key, value in cell.items() if key not in ['artifactName', 'producerBinding']}, "Ledger suite values differ from original runner artifact")
        check(actual["sourceSHA"] == source["sha"] and actual["sourceTree"] == source["tree"], "Actual semantic receipt source differs")
        expected = discovered["plans"][cell["kind"]]
        if cell['kind'] == 'examples-browser':
            check(actual.get('excludedCases', []) == expected.get('excludedCases', []), 'Runner browser exclusions differ from independent source discovery')
        check(actual.get("inventoryHash") == discovered["inventoryHash"] and actual.get("planHash") == expected["planHash"], "Runner inventory or plan hash differs from independent source discovery")
        check(actual.get("selectedIds") == expected["ids"] and actual.get("executedIds") == expected["ids"], "Runner suite selection differs from independent source discovery")
        check(actual["selectedIds"] and set(actual["selectedIds"]) == set(actual["executedIds"])
              and len(actual["selectedIds"]) == len(set(actual["selectedIds"])) == len(actual["executedIds"]), "Incomplete actual semantic inventory")
        check(actual.get("failed") == 0 and actual.get("skipped") == 0 and actual.get("executedCases", 0) > 0, "Failed or empty actual semantic suite")
        counts = actual.get("caseCounts", [])
        check(len(counts) == len(actual["selectedIds"]) and {row["id"] for row in counts} == set(actual["selectedIds"])
              and all(isinstance(row.get("executedCases"), int) and not isinstance(row["executedCases"], bool) and row["executedCases"] > 0 for row in counts)
              and sum(row["executedCases"] for row in counts) == actual["executedCases"], "Actual per-file semantic counts differ")
    return {"runId": run["id"], "runAttempt": aggregation["run_attempt"], "observedAttempt": run["run_attempt"], "artifactId": artifact["id"], "artifactDigest": artifact["digest"],
            "sourceSHA": source["sha"], "sourceTree": source["tree"], "policySHA": trusted["sha"], "acceptanceInputsHash": ledger["acceptanceInputsHash"],
            "scope": "e2e-only", "unitExecutionRequired": True, "buildBytesReused": False}
