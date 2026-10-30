"""Read-only GitHub evidence adapter. Live byte reuse remains disabled by policy."""
# cspell:words importlib urllib
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import urllib.error
import urllib.parse
import urllib.request

spec = importlib.util.spec_from_file_location("impact", Path(__file__).with_name("ci-impact-plan.py"))
impact = importlib.util.module_from_spec(spec); spec.loader.exec_module(impact)


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, file, code, message, headers, new_url):
        return None


class GitHubReadOnly:
    """No constructor network access. Never use this to retry a denied local read."""
    def __init__(self, repository, token, environment=None, allow_archive=False):
        environment = os.environ if environment is None else environment
        if environment.get("GITHUB_ACTIONS") != "true":
            raise PermissionError("Evidence transport is restricted to the authorized Actions runtime")
        if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository) or not token:
            raise ValueError("Existing repository read identity required")
        self.prefix = "https://api.github.com/repos/" + repository
        self.token = token
        self.allow_archive = allow_archive
        self.opener = urllib.request.build_opener(NoRedirect())

    def get(self, route):
        if (route and not route.startswith("/")) or ".." in route or "#" in route:
            raise ValueError("Unsafe GitHub route")
        request = urllib.request.Request(self.prefix + route, headers={"Authorization": "Bearer " + self.token, "Accept": "application/vnd.github+json", "X-GitHub-Api-Version": "2022-11-28"}, method="GET")
        try:
            with self.opener.open(request, timeout=30) as response:
                body = response.read(16 * 1024 * 1024 + 1)
                if len(body) > 16 * 1024 * 1024:
                    raise ValueError("GitHub JSON response exceeded bound")
                return json.loads(body)
        except urllib.error.HTTPError as error:
            if error.code in [401, 403]:
                raise PermissionError("GitHub evidence read denied (HTTP " + str(error.code) + ")") from None
            raise ValueError("GitHub evidence unavailable (HTTP " + str(error.code) + ")") from None

    def archive(self, artifact_id):
        if self.allow_archive is not True:
            raise PermissionError("Native reuse archive transport is disabled; build instead")
        if not isinstance(artifact_id, int) or isinstance(artifact_id, bool) or artifact_id <= 0:
            raise ValueError("Immutable artifact ID required")
        # This branch is not enabled by the current workflows. Only the reviewed
        # Actions transport may ever call it; a local denied download cannot.
        request = urllib.request.Request(self.prefix + "/actions/artifacts/" + str(artifact_id) + "/zip", headers={"Authorization": "Bearer " + self.token}, method="GET")
        try:
            response = self.opener.open(request, timeout=30)
        except urllib.error.HTTPError as error:
            if error.code in [401, 403]:
                raise PermissionError("GitHub artifact read denied (HTTP " + str(error.code) + ")") from None
            if error.code != 302:
                raise ValueError("GitHub artifact read unavailable (HTTP " + str(error.code) + ")") from None
            target = artifact_redirect_url(error.headers.get("Location", ""))
            # Authorization must never follow the signed storage redirect.
            response = self.opener.open(urllib.request.Request(target, method="GET"), timeout=30)
        with response:
            chunks, count = [], 0
            while True:
                chunk = response.read(1024 * 1024)
                if not chunk:
                    break
                count += len(chunk)
                if count > 512 * 1024 * 1024:
                    raise ValueError("Native artifact download exceeded bound")
                chunks.append(chunk)
            return b"".join(chunks)


def artifact_redirect_url(value):
    parsed = urllib.parse.urlsplit(value)
    host = parsed.hostname or ""
    if parsed.scheme != "https" or parsed.username or parsed.password or parsed.port not in [None, 443] or parsed.fragment or not (host.endswith(".blob.core.windows.net") or host.endswith(".actions.githubusercontent.com")):
        raise ValueError("Unexpected artifact storage redirect")
    return value


def committed_source(commit, expected):
    impact.revision(commit)
    files, hashes = impact.snapshot(commit)
    nodes = impact.graph(files); fingerprints = impact.fingerprints(nodes, hashes)
    owners = expected["owners"]
    if not owners or len(owners) != len(set(owners)):
        raise ValueError("Exact native source owner set required")
    inputs = {owner: fingerprints[owner]["product"] for owner in sorted(owners)}
    policies = expected["policyFiles"]
    if not policies or len(policies) != len(set(policies)):
        raise ValueError("Exact reviewed producer policy input set required")
    policy_hash = impact.digest({file: hashes[file] for file in sorted(policies)})
    workflow = expected["workflowPath"]
    return {"commitSHA": commit, "tree": impact.git("rev-parse", commit + "^{tree}").decode().strip(),
            "parents": impact.git("show", "-s", "--format=%P", commit).decode().split(),
            "sourceHash": impact.digest(inputs), "buildPolicyHash": policy_hash,
            "workflowHash": hashlib.sha256(files[workflow]).hexdigest()}


class GitHubEvidenceAdapter:
    """Adapter gets identities from the API, not artifact-provided GitHub JSON."""
    def __init__(self, transport, source_reader=committed_source):
        self.transport = transport; self.source_reader = source_reader

    def candidates(self, expected):
        # Bound runs and pages separately. Nothing scans repository history.
        workflow = urllib.parse.quote(Path(expected["workflowPath"]).name, safe="")
        runs = self.transport.get("/actions/workflows/" + workflow + "/runs?per_page=3&status=completed")["workflow_runs"]
        found = []
        for run in runs[:3]:
            for page in range(1, 4):
                response = self.transport.get("/actions/runs/" + str(run["id"]) + "/artifacts?per_page=100&page=" + str(page))
                artifacts = response["artifacts"]
                for artifact in artifacts:
                    pattern = r"native-proof-([a-f0-9]{40})-" + re.escape(expected["context"]["abi"]) + "-" + str(run["run_attempt"])
                    match = re.fullmatch(pattern, artifact["name"])
                    if match and artifact.get("expired") is False:
                        found.append({"runId": run["id"], "attempt": run["run_attempt"], "artifactId": artifact["id"], "originBuildSHA": match[1]})
                        if len(found) == 3:
                            return found
                if len(artifacts) < 100:
                    break
        return found

    def load(self, candidate, expected):
        run_id, attempt = candidate["runId"], candidate["attempt"]
        if not all(isinstance(value, int) and value > 0 for value in [run_id, attempt, candidate["artifactId"]]):
            raise ValueError("Invalid immutable GitHub identity")
        run = self.transport.get("/actions/runs/" + str(run_id) + "/attempts/" + str(attempt))
        repository = self.transport.get("")
        if repository["id"] != expected["repositoryId"]:
            raise ValueError("Repository identity differs")
        branch = self.transport.get("/branches/" + urllib.parse.quote(repository["default_branch"], safe=""))
        if branch["commit"]["sha"] != expected["trustedPolicySHA"]:
            raise ValueError("Reviewed default-branch policy moved; build instead")
        jobs = []
        for page in range(1, 4):
            batch = self.transport.get("/actions/runs/" + str(run_id) + "/attempts/" + str(attempt) + "/jobs?per_page=100&page=" + str(page))["jobs"]
            jobs.extend(batch)
            if len(batch) < 100:
                break
        selected = [job for job in jobs if job["name"] == expected["producerJobName"]]
        if len(selected) != 1:
            raise ValueError("Missing or ambiguous successful producer job")
        job = selected[0]
        if job["status"] != "completed" or job["conclusion"] != "success":
            raise ValueError("Producer failed, was cancelled or is incomplete")
        artifact = self.transport.get("/actions/artifacts/" + str(candidate["artifactId"]))
        wanted_name = "native-proof-" + candidate["originBuildSHA"] + "-" + expected["context"]["abi"] + "-" + str(attempt)
        if artifact["name"] != wanted_name or artifact["workflow_run"]["id"] != run_id or artifact["expired"]:
            raise ValueError("Wrong producer artifact or expiry")
        if not job["started_at"] <= artifact["created_at"] <= job["completed_at"]:
            raise ValueError("Artifact is not from the successful producer attempt")
        source = self.source_reader(candidate["originBuildSHA"], expected)
        reviewed = self.source_reader(expected["trustedPolicySHA"], expected)
        if source["buildPolicyHash"] != reviewed["buildPolicyHash"] or source["workflowHash"] != reviewed["workflowHash"]:
            raise ValueError("Producer policy is not the reviewed default-branch implementation")
        evidence = {"run": {"id": run["id"], "attempt": run["run_attempt"], "repositoryId": run["repository"]["id"], "headRepositoryId": run["head_repository"]["id"], "event": run["event"], "path": run["path"], "headSHA": run["head_sha"], "status": run["status"]},
                    "job": {"id": job["id"], "runId": job["run_id"], "runAttempt": attempt, "name": job["name"], "status": job["status"], "conclusion": job["conclusion"]},
                    "artifact": {"id": artifact["id"], "runId": artifact["workflow_run"]["id"], "runAttempt": attempt, "expired": artifact["expired"], "size": artifact["size_in_bytes"], "digest": artifact.get("digest")}, "sourceProof": source}
        if run["event"] == "pull_request":
            prs = run["pull_requests"]
            if len(prs) != 1:
                raise ValueError("Exact producer PR association required")
            evidence["run"]["baseSHA"] = prs[0]["base"]["sha"]
        return self.transport.archive(candidate["artifactId"]), evidence


def capture_source(products_file, output):
    """Source ledger for the actual successful producer, not a reuse grant."""
    products = json.loads(Path(products_file).read_text())
    commit = os.environ["GITHUB_SHA"]
    owners = sorted({"npm:" + row["owner"] for row in products["records"]})
    policy_files = sorted(set(impact.BUILD_POLICY + ["scripts/ci-native-capture.mjs", "scripts/ci-native-proof.py", "scripts/ci-native-evidence.py", ".github/workflows/ci.yaml", ".github/workflows/ci-primary-platform.yaml", ".github/workflows/rust-build-user.yaml", ".github/workflows/rust-build-release.yaml"]))
    expected = {"owners": owners, "policyFiles": policy_files, "workflowPath": ".github/workflows/ci.yaml"}
    source = committed_source(commit, expected)
    record = {"schemaVersion": 1, "kind": "native-source-shadow-ledger", "source": source, "owners": owners, "policyFiles": policy_files,
              "abi": products["abi"], "profile": products["profile"], "originBuildSHA": commit, "runId": os.environ["GITHUB_RUN_ID"],
              "runAttempt": os.environ["GITHUB_RUN_ATTEMPT"], "jobKey": os.environ["GITHUB_JOB"], "reuseAuthorized": False, "acceptanceInherited": False}
    destination = Path(output); destination.parent.mkdir(parents=True, exist_ok=True); destination.write_text(json.dumps(record, indent=2) + "\n")


def audit_current_run(output):
    """Uses the assembly job's existing actions:read; never downloads a ZIP."""
    record = {"schemaVersion": 1, "mode": "shadow", "reuseAuthorized": False, "acceptanceInherited": False, "producers": [], "errors": []}
    try:
        run_id, attempt = int(os.environ["GITHUB_RUN_ID"]), int(os.environ["GITHUB_RUN_ATTEMPT"])
        client = GitHubReadOnly(os.environ["GITHUB_REPOSITORY"], os.environ["GH_TOKEN"])
        run = client.get("/actions/runs/" + str(run_id) + "/attempts/" + str(attempt))
        if run["id"] != run_id or run["run_attempt"] != attempt:
            raise ValueError("Current producer attempt differs")
        jobs, artifacts = [], []
        for page in range(1, 4):
            batch = client.get("/actions/runs/" + str(run_id) + "/attempts/" + str(attempt) + "/jobs?per_page=100&page=" + str(page))["jobs"]
            jobs.extend(batch)
            if len(batch) < 100:
                break
        for page in range(1, 4):
            batch = client.get("/actions/runs/" + str(run_id) + "/artifacts?per_page=100&page=" + str(page))["artifacts"]
            artifacts.extend(batch)
            if len(batch) < 100:
                break
        for abi in impact.TARGETS:
            suffix = ("Build and Upload User Artifacts - " if abi in impact.PRIMARY else "Build and Upload Release Artifacts - ") + abi
            matches = [job for job in jobs if job["name"] == suffix or job["name"].endswith(" / " + suffix)]
            wanted = "native-shadow-" + os.environ["GITHUB_SHA"] + "-" + abi + "-" + str(attempt)
            outputs = [artifact for artifact in artifacts if artifact["name"] == wanted and not artifact["expired"]]
            if len(matches) != 1 or len(outputs) != 1:
                raise ValueError("Missing or ambiguous native shadow producer: " + abi)
            job, artifact = matches[0], outputs[0]
            if job["conclusion"] != "success" or job["status"] != "completed" or artifact["workflow_run"]["id"] != run_id:
                raise ValueError("Native shadow producer was not successful: " + abi)
            if not job["started_at"] <= artifact["created_at"] <= job["completed_at"]:
                raise ValueError("Native shadow artifact is outside the producer attempt")
            record["producers"].append({"abi": abi, "runId": run_id, "runAttempt": attempt, "jobId": job["id"], "jobName": job["name"], "artifactId": artifact["id"], "artifactDigest": artifact.get("digest"), "artifactBytes": artifact["size_in_bytes"], "contextVerified": False})
        record["status"] = "api-identities-recorded-not-reuse-authority"
    except Exception as error:
        record["status"] = "unavailable"; record["errors"].append(str(error)[:1000])
    destination = Path(output); destination.parent.mkdir(parents=True, exist_ok=True); destination.write_text(json.dumps(record, indent=2) + "\n")
    return record


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(); parser.add_argument("--products"); parser.add_argument("--audit-current-run", action="store_true"); parser.add_argument("--output", required=True)
    arguments = parser.parse_args()
    if arguments.audit_current_run:
        if audit_current_run(arguments.output)["status"] == "unavailable":
            raise SystemExit(1)
    elif arguments.products:
        capture_source(arguments.products, arguments.output)
    else:
        parser.error("Explicit products or current-run audit required")
