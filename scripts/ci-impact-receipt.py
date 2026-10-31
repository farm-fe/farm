"""Current-job execution receipt. This is data, not a signature or skip grant."""
# cspell:words importlib
import argparse
import importlib.util
import json
import os
from pathlib import Path

spec = importlib.util.spec_from_file_location("impact", Path(__file__).with_name("ci-impact-plan.py"))
impact = importlib.util.module_from_spec(spec); spec.loader.exec_module(impact)


def make_observation(cell, status, environment, event, actual_sha, actual_tree):
    """Record execution independently; only the final summary may bind a plan."""
    pr = event.get("pull_request", {})
    identity = {"headSHA": pr.get("head", {}).get("sha") or environment["GITHUB_SHA"],
                "baseSHA": pr.get("base", {}).get("sha") or event.get("inputs", {}).get("release_base"),
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
    payload = {key: value for key, value in observation.items() if key != "observationHash"}
    if observation.get("schemaVersion") != 2 or observation.get("planBound") is not False or observation.get("observationHash") != impact.digest(payload):
        raise ValueError("Independent execution observation invalid")
    for key in ["headSHA", "baseSHA", "testedSHA", "testedTree", "runId", "runAttempt", "policy", "repository"]:
        if not isinstance(observation.get(key), str) or not observation[key] or observation[key] != plan.get(key):
            raise ValueError("Observed execution differs from completed plan: " + key)
    state = {"executed-success": "success", "executed-failure": "failure", "executed-cancelled": "cancelled"}.get(observation.get("status"))
    if observation.get("acceptanceInherited") is not False or observation.get("authoritativeJobIdentityVerified") is not False:
        raise ValueError("Observation cannot inherit acceptance or self-verify authority")
    environment = {"GITHUB_RUN_ID": observation["runId"], "GITHUB_RUN_ATTEMPT": observation["runAttempt"],
                   "GITHUB_JOB": observation.get("jobKey"), "GITHUB_WORKFLOW_REF": observation.get("workflowRef"), **observation.get("runner", {})}
    if not environment["GITHUB_JOB"] or not environment["GITHUB_WORKFLOW_REF"]:
        raise ValueError("Observed workflow identity unavailable")
    result = make_receipt(plan, observation["cell"], state, environment, observation["testedSHA"], observation["testedTree"])
    return {**result, "planBound": True, "observationHash": observation["observationHash"]}


def make_receipt(plan, cell, status, environment, actual_sha, actual_tree):
    if plan.get("planHash") != impact.digest({key: value for key, value in plan.items() if key != "planHash"}):
        raise ValueError("Plan digest differs")
    if plan.get("skipAuthorized") is not False or plan.get("reuseAuthorized") is not False:
        raise ValueError("Receipt cannot authorize unreviewed incremental mode")
    if cell not in plan["executionCells"]:
        raise ValueError("Job is not an expected current matrix cell")
    if plan["testedSHA"] != actual_sha or plan["testedTree"] != actual_tree:
        raise ValueError("Actual checkout identity differs from plan")
    if str(plan["runId"]) != environment["GITHUB_RUN_ID"] or str(plan["runAttempt"]) != environment["GITHUB_RUN_ATTEMPT"]:
        raise ValueError("Receipt run attempt differs")
    state = {"success": "executed-success", "failure": "executed-failure", "cancelled": "executed-cancelled"}.get(status)
    if not state:
        raise ValueError("Actual job status is unavailable")
    result = {key: plan[key] for key in ["headSHA", "baseSHA", "testedSHA", "testedTree", "planHash", "runId", "runAttempt", "policy", "repository"]}
    return {**result, "schemaVersion": 1, "cell": cell, "status": state, "jobKey": environment["GITHUB_JOB"],
            "workflowRef": environment.get("GITHUB_WORKFLOW_REF"), "runner": {key: environment.get(key) for key in ["RUNNER_OS", "RUNNER_ARCH", "ImageOS", "ImageVersion"]},
            "acceptanceInherited": False, "authoritativeJobIdentityVerified": False}


def main():
    parser = argparse.ArgumentParser(); parser.add_argument("--output", required=True); args = parser.parse_args()
    cell = os.environ["CI_IMPACT_CELL"]
    try:
        event = json.loads(Path(os.environ["GITHUB_EVENT_PATH"]).read_text())
        sha = impact.git("rev-parse", "HEAD").decode().strip(); tree = impact.git("rev-parse", "HEAD^{tree}").decode().strip()
        result = make_observation(cell, os.environ["CI_IMPACT_JOB_STATUS"], os.environ, event, sha, tree)
    except Exception as error:
        result = {"schemaVersion": 1, "cell": cell, "status": "unavailable", "error": str(error)[:1000], "authoritativeJobIdentityVerified": False}
    output = Path(args.output); output.parent.mkdir(parents=True, exist_ok=True); output.write_text(json.dumps(result, indent=2) + "\n")
    if result["status"] == "unavailable":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
