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
        check(member in ["ts.json", "browser.json", "result.json"], "Unexpected execution receipt member")
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
    check(artifact.get("name") == "farm-ci-execution-baseline-" + str(run["id"]) + "-" + str(run["run_attempt"])
          and artifact["workflow_run"]["id"] == run["id"] and artifact["expired"] is False, "Execution ledger artifact origin differs")
    expected_run = {"repositoryId": repository["id"], "runId": run["id"], "runAttempt": run["run_attempt"], "event": run["event"]}
    check(ledger.get("run") == expected_run, "Execution attempt differs")
    check(sorted(ledger.get("requiredJobs", [])) == sorted(required_jobs), "Trusted full acceptance inventory differs")
    for name in required_jobs:
        job = evidence.successful_job(jobs, name)
        check(job["run_id"] == run["id"], "Acceptance job belongs to another run")
    producer = evidence.successful_job(jobs, PRODUCER)
    upload_binding(producer, UPLOAD, artifact)
    cells = ledger.get("coverage", {}).get("cells", [])
    expected_cells = {kind + ":" + abi for kind in ["ts", "examples-browser"] for abi in PLATFORMS}
    check(len(cells) == 6 and {row["kind"] + ":" + row["abi"] for row in cells} == expected_cells, "Missing three-platform semantic coverage")
    discovered = suite_discovery(source, trusted)
    check(discovered["sourceIdentity"]["commit"] == source["sha"] and discovered["sourceIdentity"]["tree"] == source["tree"], "Independent suite source differs")
    for cell in cells:
        label, filename, job_label, step = ("ts", "ts.json", "Typescript Test", "Preserve actual TypeScript discovery and execution counts") if cell["kind"] == "ts" else ("browser", "browser.json", "Examples Test", "Preserve actual browser discovery and execution counts")
        name = "farm-ci-suite-" + label + "-" + source["sha"] + "-" + cell["abi"] + "-" + str(run["run_attempt"])
        actual, meta = fetch_json_artifact(transport, run, name, filename)
        check(actual == {key: value for key, value in cell.items() if key != "artifactName"}, "Ledger suite values differ from original runner artifact")
        job = evidence.successful_job(jobs, job_label + " (" + PLATFORMS[cell["abi"]] + ")")
        upload_binding(job, step, meta)
        check(actual["run"] == expected_run and actual["sourceSHA"] == source["sha"] and actual["sourceTree"] == source["tree"], "Actual semantic receipt source differs")
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
    return {"runId": run["id"], "runAttempt": run["run_attempt"], "artifactId": artifact["id"], "artifactDigest": artifact["digest"],
            "sourceSHA": source["sha"], "sourceTree": source["tree"], "policySHA": trusted["sha"], "acceptanceInputsHash": ledger["acceptanceInputsHash"],
            "scope": "e2e-only", "unitExecutionRequired": True, "buildBytesReused": False}
