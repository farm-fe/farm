"""Current-job execution receipt. This is data, not a signature or skip grant."""
# cspell:words importlib errno winerror lineno colno charmap
import argparse
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys

spec = importlib.util.spec_from_file_location("impact", Path(__file__).with_name("ci-impact-plan.py"))
impact = importlib.util.module_from_spec(spec); spec.loader.exec_module(impact)

IDENTITY_FIELDS = ["headSHA", "baseSHA", "testedSHA", "testedTree", "runId", "runAttempt", "policy", "repository"]
BINDING_CODES = ["observation-shape", "observation-digest", "identity-mismatch", "observation-authority", "workflow-missing", "plan-digest", "plan-policy", "unexpected-cell", "checkout-mismatch", "attempt-mismatch", "status-unavailable", "plan-shape", "invalid-identity", "external-identity-mismatch", "matrix-inventory-mismatch", "receipt-inventory-mismatch", "cell-identity-mismatch", "cell-outcome-rejected"]


class BindingError(ValueError):
    def __init__(self, code, field=None):
        self.code = code if code in BINDING_CODES else "validation-failed"
        self.field = field if field in IDENTITY_FIELDS else None
        self.cell = None
        self.counts = {}
        super().__init__(self.code)


def safe_context(environment):
    result = {}
    for source, target, pattern in [("CI_IMPACT_CELL", "cell", r"[a-z0-9:.-]{1,160}"), ("GITHUB_RUN_ID", "runId", r"[1-9][0-9]{0,24}"), ("GITHUB_RUN_ATTEMPT", "runAttempt", r"[1-9][0-9]{0,8}")]:
        value = environment.get(source)
        result[target] = value if isinstance(value, str) and re.fullmatch(pattern, value) else None
    return result


def safe_error_fields(source):
    allowed = {
        "stage": ["read-event", "git-head", "git-tree", "validate-identity", "write-receipt", "read-plan", "read-observations", "bind-observations", "validate-inventory"],
        "class": ["UnicodeDecodeError", "JSONDecodeError", "CalledProcessError", "TimeoutExpired", "FileNotFoundError", "PermissionError", "OSError", "ValueError", "BindingError", "KeyError", "TypeError"],
        "code": ["operation-failed", "invalid-text-encoding", "invalid-json", "git-command-failed", "git-command-timeout", "file-operation-failed", "validation-failed", *BINDING_CODES],
        "encoding": ["utf-8", "utf8", "ascii", "charmap", "cp1252", "cp936"],
    }
    result = {key: source[key] if source.get(key) in values else "unknown" for key, values in allowed.items() if key in source}
    if source.get("identityField") in IDENTITY_FIELDS:
        result["identityField"] = source["identityField"]
    for key in ["start", "end", "line", "column", "errno", "windowsError", "returnCode", "missingCount", "unexpectedCount", "duplicateCount"]:
        value = source.get(key)
        if type(value) is int and abs(value) <= 2 ** 53:
            result[key] = value
    if source.get("reason") in ["invalid start byte", "invalid continuation byte", "unexpected end of data", "character maps to <undefined>", "decode failed"]:
        result["reason"] = source["reason"]
    return result


def report_error(stage, error):
    """Log bounded failure metadata without event text, bytes or environment."""
    detail = {**safe_context(os.environ), "stage": stage, "class": type(error).__name__, "code": "operation-failed"}
    if isinstance(error, BindingError):
        detail.update({"code": error.code, "identityField": error.field, **error.counts})
    elif isinstance(error, UnicodeDecodeError):
        reasons = ["invalid start byte", "invalid continuation byte", "unexpected end of data", "character maps to <undefined>"]
        detail.update({"code": "invalid-text-encoding", "encoding": error.encoding, "reason": error.reason if error.reason in reasons else "decode failed", "start": error.start, "end": error.end})
    elif isinstance(error, json.JSONDecodeError):
        detail.update({"code": "invalid-json", "line": error.lineno, "column": error.colno})
    elif isinstance(error, subprocess.CalledProcessError):
        detail.update({"code": "git-command-failed", "returnCode": error.returncode})
    elif isinstance(error, subprocess.TimeoutExpired):
        detail["code"] = "git-command-timeout"
    elif isinstance(error, OSError):
        detail.update({"code": "file-operation-failed", "errno": error.errno, "windowsError": getattr(error, "winerror", None)})
    elif isinstance(error, ValueError):
        detail["code"] = "validation-failed"
    detail = {**safe_context(os.environ), **safe_error_fields(detail)}
    if isinstance(error, BindingError) and error.cell in impact.execution_cells("unit") + impact.execution_cells("e2e"):
        detail["cell"] = error.cell
    sys.stderr.write("CI impact diagnostic: " + json.dumps(detail, ensure_ascii=True) + "\n")
    return detail


def make_observation(cell, status, environment, event, actual_sha, actual_tree):
    """Record execution independently; only the final summary may bind a plan."""
    pr = event.get("pull_request", {})
    identity = {"headSHA": pr.get("head", {}).get("sha") or environment["GITHUB_SHA"],
                "baseSHA": pr.get("base", {}).get("sha") or event.get("inputs", {}).get("base_sha") or event.get("inputs", {}).get("release_base") or event.get("before"),
                "testedSHA": actual_sha, "testedTree": actual_tree, "runId": environment["GITHUB_RUN_ID"],
                "runAttempt": environment["GITHUB_RUN_ATTEMPT"], "repository": environment["GITHUB_REPOSITORY"], "policy": impact.POLICY}
    if actual_sha != environment["GITHUB_SHA"] or not cell or not all(isinstance(value, str) and value for value in identity.values()):
        raise ValueError("Independent execution identity unavailable or checkout differs")
    state = {"success": "executed-success", "failure": "executed-failure", "cancelled": "executed-cancelled"}.get(status)
    if not state or not environment.get("GITHUB_JOB") or not environment.get("GITHUB_WORKFLOW_REF"):
        raise ValueError("Actual job outcome or workflow identity unavailable")
    result = {**identity, "schemaVersion": 2, "cell": cell, "status": state, "planBound": False,
              "jobKey": environment["GITHUB_JOB"], "workflowRef": environment["GITHUB_WORKFLOW_REF"],
              "runner": {key: environment.get(key) for key in ["RUNNER_OS", "RUNNER_ARCH", "ImageOS", "ImageVersion"]},
              "acceptanceInherited": False, "authoritativeJobIdentityVerified": False}
    return {**result, "observationHash": impact.digest(result)}


def bind_observation(plan, observation):
    """Bind after planning finishes; this hash is not a signature or trust grant."""
    if not isinstance(observation, dict) or observation.get("schemaVersion") != 2 or observation.get("planBound") is not False:
        raise BindingError("observation-shape")
    payload = {key: value for key, value in observation.items() if key != "observationHash"}
    if observation.get("observationHash") != impact.digest(payload):
        raise BindingError("observation-digest")
    for key in IDENTITY_FIELDS:
        if not isinstance(observation.get(key), str) or not observation[key] or observation[key] != plan.get(key):
            raise BindingError("identity-mismatch", key)
    state = {"executed-success": "success", "executed-failure": "failure", "executed-cancelled": "cancelled"}.get(observation.get("status"))
    if observation.get("acceptanceInherited") is not False or observation.get("authoritativeJobIdentityVerified") is not False:
        raise BindingError("observation-authority")
    environment = {"GITHUB_RUN_ID": observation["runId"], "GITHUB_RUN_ATTEMPT": observation["runAttempt"],
                   "GITHUB_JOB": observation.get("jobKey"), "GITHUB_WORKFLOW_REF": observation.get("workflowRef"), **observation.get("runner", {})}
    if not environment["GITHUB_JOB"] or not environment["GITHUB_WORKFLOW_REF"]:
        raise BindingError("workflow-missing")
    result = make_receipt(plan, observation["cell"], state, environment, observation["testedSHA"], observation["testedTree"])
    return {**result, "planBound": True, "observationHash": observation["observationHash"]}


def make_receipt(plan, cell, status, environment, actual_sha, actual_tree):
    if plan.get("planHash") != impact.digest({key: value for key, value in plan.items() if key != "planHash"}):
        raise BindingError("plan-digest")
    if plan.get("skipAuthorized") is not False or plan.get("reuseAuthorized") is not False:
        raise BindingError("plan-policy")
    if cell not in plan["executionCells"]:
        raise BindingError("unexpected-cell")
    if plan["testedSHA"] != actual_sha or plan["testedTree"] != actual_tree:
        raise BindingError("checkout-mismatch")
    if str(plan["runId"]) != environment["GITHUB_RUN_ID"] or str(plan["runAttempt"]) != environment["GITHUB_RUN_ATTEMPT"]:
        raise BindingError("attempt-mismatch")
    state = {"success": "executed-success", "failure": "executed-failure", "cancelled": "executed-cancelled"}.get(status)
    if not state:
        raise BindingError("status-unavailable")
    result = {key: plan[key] for key in ["headSHA", "baseSHA", "testedSHA", "testedTree", "planHash", "runId", "runAttempt", "policy", "repository"]}
    return {**result, "schemaVersion": 1, "cell": cell, "status": state, "jobKey": environment["GITHUB_JOB"],
            "workflowRef": environment.get("GITHUB_WORKFLOW_REF"), "runner": {key: environment.get(key) for key in ["RUNNER_OS", "RUNNER_ARCH", "ImageOS", "ImageVersion"]},
            "acceptanceInherited": False, "authoritativeJobIdentityVerified": False}


def main():
    parser = argparse.ArgumentParser(); parser.add_argument("--output", required=True); args = parser.parse_args()
    cell = os.environ["CI_IMPACT_CELL"]
    operation = "read-event"
    try:
        event = json.loads(Path(os.environ["GITHUB_EVENT_PATH"]).read_text(encoding="utf-8"))
        operation = "git-head"
        sha = impact.git("rev-parse", "HEAD").decode("ascii").strip()
        operation = "git-tree"
        tree = impact.git("rev-parse", "HEAD^{tree}").decode("ascii").strip()
        operation = "validate-identity"
        result = make_observation(cell, os.environ["CI_IMPACT_JOB_STATUS"], os.environ, event, sha, tree)
    except Exception as error:
        result = {"schemaVersion": 1, "cell": cell, "status": "unavailable", "error": report_error(operation, error), "authoritativeJobIdentityVerified": False}
    try:
        output = Path(args.output); output.parent.mkdir(parents=True, exist_ok=True); output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    except Exception as error:
        report_error("write-receipt", error)
        raise SystemExit(1) from None
    if result["status"] == "unavailable":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
