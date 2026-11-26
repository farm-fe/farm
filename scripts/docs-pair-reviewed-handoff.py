"""Read-only contract for handing original Docs archives to the manual writer.

Callbacks stand for separately reviewed, repository-pinned GET transports and
a digest-pinned artifact download. This module has no transport, real runner
entry point, credentials, publishing, or permission changes.
"""
# cspell:words zipfile crc decompress
import base64
import hashlib
import io
import json
import re
import stat
import zipfile


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def positive_id(value):
    return type(value) is int and 0 < value < 10**20


def sha(value, size):
    return isinstance(value, str) and re.fullmatch(r"[a-f0-9]{%d}" % size, value) is not None


PACKAGE_ORDER = ("@farmfe/docs-theme-default", "@farmfe/docs")
REPOSITORY = "farm-fe/farm"
REPOSITORY_ID = 507542208
PRODUCER_ID = 428986974
PRODUCER = ".github/workflows/docs-pair-bootstrap.yaml"
# The live registration binds the writer workflow ID for each run.
WRITER = ".github/workflows/docs-pair-publish.yaml"
JOB = "Build and verify frozen Docs pair"
UPLOAD = "Upload exact original Docs archives"
MAX_TAR = 32 * 1024**2
MAX_ZIP = 2 * MAX_TAR + 1024**2
RECEIPT = "frozen-pair.json"
TARS = {name: "farmfe-" + name.split("/")[1] + "-2.0.0.tgz" for name in PACKAGE_ORDER}
SELECTION = {"sourceSHA", "sourceTree", "producerRun", "writerRun", "artifactId",
             "artifactSha256", "tag"}


def exact_repo(value):
    return type(value) is dict and value.get("id") == REPOSITORY_ID and value.get("full_name") == REPOSITORY


def require_main(get, sha, tree):
    commit = get(f"/git/commits/{sha}")
    require(commit.get("sha") == sha and commit.get("tree", {}).get("sha") == tree, "Exact source tree differs")
    branch = get("/branches/main")
    require(branch.get("name") == "main" and branch.get("protected") is True
            and branch.get("commit", {}).get("sha") == sha, "Exact protected main no longer current")


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
        require(type(actor) is dict and positive_id(actor.get("id"))
                and (producer or actor.get("type") == "User"), "Exact initiating actor required")
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
    require(positive_id(job.get("id")) and job.get("run_id") == run_id
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


def require_first_writer(get, run_id, writer_workflow_id):
    # A failed, cancelled or absent prior response is not evidence that no
    # write happened. Any other record, or incomplete history, stops this lane.
    listing = get(f"/actions/workflows/{writer_workflow_id}/runs?branch=main&event=workflow_dispatch&per_page=2")
    runs = listing.get("workflow_runs")
    require(type(listing.get("total_count")) is int and listing["total_count"] == 1
            and type(runs) is list and len(runs) == 1 and type(runs[0]) is dict
            and runs[0].get("id") == run_id and runs[0].get("run_attempt") == 1,
            "Prior or ambiguous publisher history; stop for read-only reconciliation")


def read_reviewed_handoff(selection, actor_id, *, get, download):
    """Check fixed objects under one explicit dispatch; not a global lock."""
    require(type(selection) is dict and set(selection) == SELECTION and positive_id(actor_id),
            "Independent source selection and exact initiating actor required")
    selection = dict(selection)
    require(all(positive_id(n) for n in (selection["producerRun"], selection["writerRun"], selection["artifactId"]))
            and selection["producerRun"] != selection["writerRun"], "Producer and writer roles cannot alias")
    source_sha, tree = selection["sourceSHA"], selection["sourceTree"]
    require(sha(source_sha, 40) and sha(tree, 40) and sha(selection["artifactSha256"], 64)
            and selection["tag"] == "farm-docs-bootstrap", "Unreviewed source, artifact digest or tag")
    require_main(get, source_sha, tree)
    workflow = get("/actions/workflows/docs-pair-publish.yaml")
    writer_id = workflow.get("id")
    require(positive_id(writer_id) and writer_id != PRODUCER_ID
            and workflow.get("path") == WRITER and workflow.get("state") == "active",
            "Exact registered writer workflow required")
    exact_run(get, selection["producerRun"], PRODUCER_ID, PRODUCER, source_sha, producer=True)
    actor = exact_run(get, selection["writerRun"], writer_id, WRITER, source_sha, producer=False)
    require(actor == actor_id, "Manual dispatch initiating actor differs")
    require_first_writer(get, selection["writerRun"], writer_id)
    job_id = producer_job(get, selection["producerRun"], source_sha)
    artifact_id = selection["artifactId"]
    artifact = get(f"/actions/artifacts/{artifact_id}")
    origin = artifact.get("workflow_run", {})
    require(artifact.get("id") == artifact_id and artifact.get("name") == f"farm-docs-pair-{source_sha}-1"
            and artifact.get("expired") is False and artifact.get("digest") == "sha256:" + selection["artifactSha256"]
            and all(origin.get(k) == v for k, v in {"id": selection["producerRun"], "repository_id": REPOSITORY_ID,
                    "head_repository_id": REPOSITORY_ID, "head_branch": "main", "head_sha": source_sha}.items()),
            "Artifact metadata is not bound to the exact original producer")
    receipt, bodies = inspect_artifact(download(artifact_id, MAX_ZIP), selection)
    require_main(get, source_sha, tree)
    require_first_writer(get, selection["writerRun"], writer_id)
    return {"producerJobId": job_id, "writerWorkflowId": writer_id, "artifactId": artifact_id, "receipt": receipt, "archives": bodies}
