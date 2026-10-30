# cspell:words importlib
"""Stable shadow report. This is not publication authorization."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re

spec = importlib.util.spec_from_file_location("impact", Path(__file__).with_name("ci-impact-plan.py"))
impact = importlib.util.module_from_spec(spec); spec.loader.exec_module(impact)
receipt_spec = importlib.util.spec_from_file_location("receipt", Path(__file__).with_name("ci-impact-receipt.py"))
receipt = importlib.util.module_from_spec(receipt_spec); receipt_spec.loader.exec_module(receipt)


def validate_cells(plan, receipts, *, trusted_inventory, expected_identity, require_success=True):
    """Future stable gate primitive; missing matrix cells never become success.

    Caller must obtain receipts through the trusted current-run evidence adapter.
    A reusable workflow's coarse needs.result cannot provide this evidence.
    """
    if not isinstance(plan, dict) or not isinstance(receipts, list) or not isinstance(expected_identity, dict):
        raise ValueError("Typed plan, receipts and external run identity required")
    if not all(isinstance(row, dict) and isinstance(row.get("cell"), str) and row["cell"] for row in receipts):
        raise ValueError("Receipt cell identity missing")
    payload = {key: value for key, value in plan.items() if key != "planHash"}
    actual_hash = hashlib.sha256(json.dumps(payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode()).hexdigest()
    if actual_hash != plan.get("planHash"):
        raise ValueError("Plan digest differs")
    if not isinstance(trusted_inventory, (list, tuple)) or not trusted_inventory or not all(isinstance(cell, str) and cell for cell in trusted_inventory) or len(trusted_inventory) != len(set(trusted_inventory)):
        raise ValueError("Nonempty reviewed matrix inventory required")
    identity = ["headSHA", "baseSHA", "testedSHA", "testedTree", "planHash", "runId", "runAttempt", "policy", "repository"]
    for key in identity:
        value = plan.get(key)
        if not isinstance(value, str) or not value:
            raise ValueError("Required plan identity missing: " + key)
        if key.endswith("SHA") or key == "testedTree":
            if not re.fullmatch(r"[a-f0-9]{40}", value):
                raise ValueError("Invalid Git identity: " + key)
        if key in ["runId", "runAttempt"] and not re.fullmatch(r"[1-9][0-9]*", value):
            raise ValueError("Invalid run attempt identity")
        if value != expected_identity.get(key):
            raise ValueError("External run identity differs: " + key)
    wanted = plan.get("executionCells")
    if not isinstance(wanted, list) or not wanted or not all(isinstance(cell, str) and cell for cell in wanted) or len(wanted) != len(set(wanted)) or set(wanted) != set(trusted_inventory):
        raise ValueError("Plan matrix differs from reviewed policy")
    found = [row["cell"] for row in receipts]
    if len(found) != len(set(found)) or set(found) != set(wanted):
        raise ValueError("Missing, duplicate or unexpected matrix cell")
    for receipt in receipts:
        for key in identity:
            if key not in receipt or receipt[key] != plan[key]:
                raise ValueError("Cell identity differs: " + key)
        allowed = ["executed-success"] if require_success else ["executed-success", "executed-failure", "executed-cancelled"]
        if receipt.get("status") not in allowed or receipt.get("acceptanceInherited") is not False:
            raise ValueError("Acceptance was skipped, cancelled, failed or inherited")
    return True


def summarize(plan, needs, expected_jobs, informational_jobs=("performance-report", "performance-summary")):
    if plan.get("mode") != "shadow" or plan.get("skipAuthorized") is not False or plan.get("reuseAuthorized") is not False:
        raise ValueError("Unreviewed incremental execution policy")
    rows = []
    for name in expected_jobs:
        result = needs.get(name, {}).get("result", "missing")
        rows.append({"job": name, "result": result, "verified": result == "success"})
    required = [row for row in rows if row["job"] not in informational_jobs]
    informational = [row for row in rows if row["job"] in informational_jobs]
    return {"schemaVersion": 1, "mode": "shadow", "planHash": plan.get("planHash"), "jobs": rows,
            "identity": {key: plan.get(key) for key in ["baseSHA", "headSHA", "testedSHA", "testedTree", "policy", "runId", "runAttempt"]},
            "allExpectedJobsSuccessful": all(row["verified"] for row in required),
            "requiredFailures": [row for row in required if not row["verified"]], "informationalOutcomes": informational,
            "publicationAuthorized": False, "dynamicMatrixVerified": False,
            "note": "Reusable job results are not per-cell proof. Existing exact release gates remain authoritative."}


def cell_outcomes(rows):
    outcomes = [{"cell": row["cell"], "status": row["status"]} for row in rows]
    required_failures = [row for row in outcomes if not row["cell"].startswith("performance:") and row["cell"] != "performance-summary" and row["status"] != "executed-success"]
    return outcomes, required_failures


def read_observations(directory):
    directory = Path(directory).resolve(strict=True); rows = []; unavailable = []; total = 0
    for file in directory.rglob("*.json"):
        if file.is_symlink() or not file.resolve(strict=True).is_relative_to(directory):
            raise ValueError("Receipt path redirects outside artifact input")
        total += file.stat().st_size
        if total > 1024 * 1024 or len(rows) >= 64:
            raise ValueError("Receipt input exceeds bound")
        observed = json.loads(file.read_text(encoding="utf-8"))
        rows.append(observed)
        if observed.get("status") == "unavailable":
            source = observed.get("error", {})
            detail = receipt.safe_error_fields(source) if isinstance(source, dict) else {}
            unavailable.append({**receipt.safe_context({"CI_IMPACT_CELL": observed.get("cell"), "GITHUB_RUN_ID": source.get("runId") if isinstance(source, dict) else None, "GITHUB_RUN_ATTEMPT": source.get("runAttempt") if isinstance(source, dict) else None}), "error": detail})
    return rows, unavailable


def main():
    parser = argparse.ArgumentParser(); parser.add_argument("--plan", required=True); parser.add_argument("--receipts"); parser.add_argument("--output", required=True); args = parser.parse_args()
    try:
        plan = json.loads(Path(args.plan).read_text(encoding="utf-8"))
    except (OSError, ValueError) as error:
        plan = {"mode": "shadow", "skipAuthorized": False, "reuseAuthorized": False, "planHash": None, "unavailable": receipt.report_error("read-plan", error)}
    needs = json.loads(os.environ["CI_IMPACT_NEEDS"])
    try:
        event = json.loads(Path(os.environ["GITHUB_EVENT_PATH"]).read_text(encoding="utf-8"))
    except (OSError, ValueError, KeyError):
        event = {}
    # No selected task may be skipped in shadow execution. Seal is conditional on
    # the existing release scope and is accounted for by the publication gate.
    expected = ["impact-plan", "release-contract-tests", "performance-contract-tests", "call-rust-build-user", "call-rust-build-release", "check-native-candidate-assembly", "examples-test", "docs-acceptance", "type-check", "ts-test", "performance-report", "performance-summary"]
    if os.environ.get("CI_IMPACT_SCOPE") == "unit":
        expected = ["impact-plan", "cargo-check", "cargo-test"]
    elif impact.release_candidate(event, os.environ):
        expected.append("release-candidate-seal")
    report = summarize(plan, needs, expected)
    report["planUnavailable"] = plan.get("unavailable")
    if report["planUnavailable"]:
        report["allExpectedJobsSuccessful"] = False
    report["cellBindingsVerified"] = False; report["cellOutcomes"] = []
    report["requiredCellFailures"] = []; report["allRequiredCellsSuccessful"] = False
    operation = "read-event"
    try:
        event = json.loads(Path(os.environ["GITHUB_EVENT_PATH"]).read_text(encoding="utf-8"))
        operation = "git-tree"
        actual_tree = impact.git("rev-parse", "HEAD^{tree}").decode("ascii").strip()
        operation = "validate-identity"
        pr = event.get("pull_request", {})
        identity = {"headSHA": pr.get("head", {}).get("sha") or os.environ["GITHUB_SHA"], "baseSHA": pr.get("base", {}).get("sha") or event.get("inputs", {}).get("release_base"),
                    "testedSHA": os.environ["GITHUB_SHA"], "testedTree": actual_tree, "runId": os.environ["GITHUB_RUN_ID"],
                    "runAttempt": os.environ["GITHUB_RUN_ATTEMPT"], "repository": os.environ["GITHUB_REPOSITORY"], "policy": impact.POLICY, "planHash": plan.get("planHash")}
        candidate = impact.release_candidate(event, os.environ)
        inventory = impact.execution_cells(os.environ.get("CI_IMPACT_SCOPE", "e2e"), candidate)
        if not args.receipts:
            raise ValueError("Current-run matrix receipts unavailable")
        operation = "read-observations"
        observations, report["unavailableCells"] = read_observations(args.receipts)
        if report["unavailableCells"]:
            raise ValueError("Current cells have unavailable execution identity")
        operation = "bind-observations"
        rows = [receipt.bind_observation(plan, observed) for observed in observations]
        operation = "validate-inventory"
        validate_cells(plan, rows, trusted_inventory=inventory, expected_identity=identity, require_success=False)
        report["cellBindingsVerified"] = True
        report["cellOutcomes"], report["requiredCellFailures"] = cell_outcomes(rows)
        report["allRequiredCellsSuccessful"] = not report["requiredCellFailures"]
    except Exception as error:
        report["cellBindingsError"] = receipt.report_error(operation, error)
    report["allExpectedJobsSuccessful"] = report["allExpectedJobsSuccessful"] and report["cellBindingsVerified"] and report["allRequiredCellsSuccessful"]
    output = Path(args.output); output.parent.mkdir(parents=True, exist_ok=True); output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with open(summary, "a", encoding="utf-8") as stream:
            stream.write("### Incremental CI shadow plan\n\nNo tasks were omitted; no prior acceptance result was inherited.\n\n")
            stream.write("| Job | Result |\n|---|---|\n" + "".join("| " + row["job"] + " | " + row["result"] + " |\n" for row in report["jobs"]))
    if not report["allExpectedJobsSuccessful"]:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
