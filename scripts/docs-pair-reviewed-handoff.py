"""Offline contract for handing original Docs archives to a reviewed role.

Callbacks stand for separately reviewed, repository-pinned GET transports and
a digest-pinned artifact download. This module has no transport, real runner
entry point, approval action, credentials, publishing, or release state.
"""
# cspell:words importlib zipfile crc decompress
import base64
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import stat
import zipfile


spec = importlib.util.spec_from_file_location(
    "docs_pair_owner_approval", Path(__file__).with_name("docs-pair-owner-approval.py")
)
owner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(owner)
require = owner.require
REPOSITORY_ID = 507542208
PRODUCER_ID = 428986974
ENVIRONMENT_NAME = "docs-npm-bootstrap"
PRODUCER = ".github/workflows/docs-pair-bootstrap.yaml"
# Proposed writer identity only. No such workflow or registered ID is assumed.
WRITER = ".github/workflows/docs-pair-publish.yaml"
JOB = "Build and verify frozen Docs pair"
UPLOAD = "Upload exact original Docs archives"
MAX_TAR = 32 * 1024**2
MAX_ZIP = 2 * MAX_TAR + 1024**2
RECEIPT = "frozen-pair.json"
TARS = {name: "farmfe-" + name.split("/")[1] + "-2.0.0.tgz" for name in owner.PACKAGE_ORDER}
POLICY = {"producerWorkflowId", "writerWorkflowId", "environmentId",
          "environmentName", "trustedReviewerIds"}
SELECTION = {"sourceSHA", "sourceTree", "producerRun", "writerRun", "artifactId",
             "artifactSha256", "tag"}


def exact_repo(value):
    return type(value) is dict and value.get("id") == REPOSITORY_ID and value.get("full_name") == owner.REPOSITORY


def require_main(get, sha, tree):
    branch = get("/branches/main")
    require(branch.get("name") == "main" and branch.get("protected") is True
            and branch.get("commit", {}).get("sha") == sha, "Exact protected main no longer current")
    commit = get(f"/git/commits/{sha}")
    require(commit.get("sha") == sha and commit.get("tree", {}).get("sha") == tree, "Exact source tree differs")


def exact_run(get, run_id, workflow_id, path, sha, *, producer):
    expected = {"id": run_id, "workflow_id": workflow_id, "path": path,
                "event": "workflow_dispatch", "head_branch": "main", "head_sha": sha,
                "run_attempt": 1, "status": "completed" if producer else "in_progress",
                "conclusion": "success" if producer else None}
    for route in (f"/actions/runs/{run_id}", f"/actions/runs/{run_id}/attempts/1"):
        observed = get(route)
        require(type(observed) is dict and all(k in observed and type(observed[k]) is type(v)
                    and observed[k] == v for k, v in expected.items())
                and exact_repo(observed.get("repository")) and exact_repo(observed.get("head_repository")),
                "Exact canonical workflow, repository and first release attempt required")
        actor = observed.get("actor")
        require(type(actor) is dict and owner.positive_id(actor.get("id")), "Exact initiating actor required")
        if route.endswith("/attempts/1"):
            require(actor["id"] == actor_id, "Initiator differs across run records")
        actor_id = actor["id"]
    return actor_id


def producer_job(get, run_id, sha):
    page = get(f"/actions/runs/{run_id}/attempts/1/jobs")
    rows = page.get("jobs")
    require(type(rows) is list and 0 < len(rows) <= 30 and page.get("total_count") == len(rows)
            and all(type(job) is dict for job in rows), "Incomplete or unknown producer job list")
    jobs = [j for j in rows if j.get("name") == JOB]
    require(len(jobs) == 1, "Exact original producer job required")
    job = jobs[0]
    require(owner.positive_id(job.get("id")) and job.get("run_id") == run_id
            and type(job.get("run_attempt", 1)) is int and job.get("run_attempt", 1) == 1
            and job.get("head_sha") == sha
            and job.get("status") == "completed" and job.get("conclusion") == "success",
            "Original producer job was not proven successful")
    steps = job.get("steps")
    require(type(steps) is list and 0 < len(steps) <= 64 and all(type(s) is dict for s in steps),
            "Incomplete producer steps")
    uploads = [s for s in steps if s.get("name") == UPLOAD]
    require(len(uploads) == 1 and uploads[0].get("status") == "completed"
            and uploads[0].get("conclusion") == "success", "Original immutable upload not proven")
    return job["id"]


def receipt_bytes(body):
    def no_duplicates(items):
        out = dict(items)
        require(len(out) == len(items), "Duplicate frozen receipt key")
        return out
    return json.loads(body, object_pairs_hook=no_duplicates)


def inspect_artifact(data, selection):
    require(type(data) is bytes and 0 < len(data) <= MAX_ZIP
            and hashlib.sha256(data).hexdigest() == selection["artifactSha256"], "Artifact ZIP bytes differ")
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        entries = archive.infolist()
        require(len(entries) == 3 and {entry.filename for entry in entries} == {RECEIPT, *TARS.values()},
                "Artifact must contain only the exact original files")
        for entry in entries:
            require(stat.S_IFMT(entry.external_attr >> 16) in {0, stat.S_IFREG} and not entry.is_dir()
                    and 0 < entry.file_size < (32768 if entry.filename == RECEIPT else MAX_TAR),
                    "Original artifact has an unexpected member type or size")
        receipt = receipt_bytes(archive.read(RECEIPT))
        bodies = {name: archive.read(path) for name, path in TARS.items()}
    require(type(receipt) is dict and set(receipt) == {"schemaVersion", "kind", "sourceSHA", "sourceTree", "packages"}
            and type(receipt["schemaVersion"]) is int and receipt["schemaVersion"] == 1
            and receipt["kind"] == "docs-frozen-pair-v1" and receipt["sourceSHA"] == selection["sourceSHA"]
            and receipt["sourceTree"] == selection["sourceTree"], "Frozen receipt source differs")
    packages = receipt["packages"]
    require(type(packages) is list and len(packages) == 2, "Exact frozen package pair required")
    for name, package in zip(TARS, packages):
        data = bodies[name]
        observed = {"name": name, "version": "2.0.0", "archiveBytes": len(data),
                    "archiveSha256": hashlib.sha256(data).hexdigest(),
                    "integrity": "sha512-" + base64.b64encode(hashlib.sha512(data).digest()).decode("ascii")}
        require(type(package) is dict and type(package.get("archiveBytes")) is int and package == observed,
                "Frozen receipt does not match original package bytes")
    return receipt, bodies


def read_reviewed_handoff(selection, policy, *, get, download):
    """Check a proposed release once; success is not a durable cross-run claim."""
    require(type(selection) is dict and set(selection) == SELECTION and type(policy) is dict and set(policy) == POLICY,
            "Independent source selection and reviewed policy required")
    selection = dict(selection)
    policy = dict(policy)
    p, w = policy["producerWorkflowId"], policy["writerWorkflowId"]
    require(all(owner.positive_id(n) for n in (p, w, policy["environmentId"],
                 selection["producerRun"], selection["writerRun"], selection["artifactId"]))
            and p != w and selection["producerRun"] != selection["writerRun"], "Producer and writer roles cannot alias")
    require(p == PRODUCER_ID and policy["environmentName"] == ENVIRONMENT_NAME,
            "Canonical source producer and dedicated environment required")
    sha, tree = selection["sourceSHA"], selection["sourceTree"]
    require(owner.sha(sha, 40) and owner.sha(tree, 40) and owner.sha(selection["artifactSha256"], 64)
            and selection["tag"] == "farm-docs-bootstrap", "Unreviewed source, artifact digest or tag")
    require_main(get, sha, tree)
    exact_run(get, selection["producerRun"], p, PRODUCER, sha, producer=True)
    actor_id = exact_run(get, selection["writerRun"], w, WRITER, sha, producer=False)
    job_id = producer_job(get, selection["producerRun"], sha)
    artifact_id = selection["artifactId"]
    artifact = get(f"/actions/artifacts/{artifact_id}")
    origin = artifact.get("workflow_run", {})
    require(artifact.get("id") == artifact_id and artifact.get("name") == f"farm-docs-pair-{sha}-1"
            and artifact.get("expired") is False and artifact.get("digest") == "sha256:" + selection["artifactSha256"]
            and all(origin.get(k) == v for k, v in {"id": selection["producerRun"], "repository_id": REPOSITORY_ID,
                    "head_repository_id": REPOSITORY_ID, "head_branch": "main", "head_sha": sha}.items()),
            "Artifact metadata is not bound to the exact original producer")
    receipt, bodies = inspect_artifact(download(artifact_id, MAX_ZIP), selection)
    permit = {**selection, "attempt": 1, "environmentId": policy["environmentId"],
              "archives": {row["name"]: row["archiveSha256"] for row in receipt["packages"]}}
    owner.verify_owner_review(permit, review_history=get, trusted_reviewer_ids=policy["trustedReviewerIds"],
                              actor_id=actor_id, environment_name=policy["environmentName"])
    require_main(get, sha, tree)
    return {"producerJobId": job_id, "artifactId": artifact_id, "receipt": receipt, "archives": bodies}
