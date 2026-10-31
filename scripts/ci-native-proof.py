# cspell:words IFMT IFREG casefold maketrans conin conout
"""Strict native byte proof primitives. No network, credentials or skip authority.

GitHub evidence is a separate trusted caller input, never read from the artifact.
The live workflow does not enable reuse until that adapter is reviewed.
"""
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import re
import stat
import unicodedata
import zipfile

SCHEMA = 1
MAX_ARCHIVE = 512 * 1024 * 1024
MAX_FILES = 64
CONTEXT_FIELDS = {"profile", "abi", "target", "host", "toolchain", "linker", "image", "flags", "features", "lockHash", "sourceHash", "buildPolicyHash"}
TARGETS = {"linux-x64-gnu": "x86_64-unknown-linux-gnu", "linux-x64-musl": "x86_64-unknown-linux-musl",
           "linux-arm64-gnu": "aarch64-unknown-linux-gnu", "linux-arm64-musl": "aarch64-unknown-linux-musl",
           "darwin-x64": "x86_64-apple-darwin", "darwin-arm64": "aarch64-apple-darwin",
           "win32-x64-msvc": "x86_64-pc-windows-msvc", "win32-ia32-msvc": "i686-pc-windows-msvc", "win32-arm64-msvc": "aarch64-pc-windows-msvc"}


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def check(condition, message):
    if not condition:
        raise ValueError(message)


def exact_sha(value, size=40):
    check(isinstance(value, str) and re.fullmatch(r"[a-f0-9]{" + str(size) + "}", value), "Exact source/hash identity missing")


def safe_member(value):
    check(isinstance(value, str) and value and "\\" not in value and ":" not in value, "Unsafe artifact member")
    item = PurePosixPath(value)
    check(not item.is_absolute() and item.as_posix() == value and unicodedata.normalize("NFC", value) == value, "Artifact member is not canonical")
    check(not any(unicodedata.category(character) in ["Cc", "Cf", "Cs", "Zl", "Zp"] for character in value), "Artifact control character")
    reserved = {"con", "prn", "aux", "nul", "conin$", "conout$", "clock$", *["com" + str(index) for index in range(1, 10)], *["lpt" + str(index) for index in range(1, 10)]}
    for part in value.split("/"):
        check(part and part not in [".", ".."] and part.rstrip(" .") == part, "Artifact path traversal or ambiguous suffix")
        stem = part.split(".")[0].casefold().translate(str.maketrans({"¹": "1", "²": "2", "³": "3"}))
        check(stem not in reserved, "Windows reserved artifact member")
    return value


def validate_member_set(members, abi):
    check(abi in TARGETS, "Known artifact target required")
    seen = set()
    for member in members:
        value = safe_member(member)
        key = value.casefold() if abi.startswith(("win32-", "darwin-")) else value
        check(key not in seen, "Artifact target filesystem collision")
        seen.add(key)


def validate_context(context):
    check(set(context) == CONTEXT_FIELDS, "Incomplete or unknown native context")
    check(context["profile"] in ["ci", "ci-test", "release"], "Native profiles are distinct")
    check(context["abi"] in TARGETS and context["target"] == TARGETS[context["abi"]], "Unsupported ABI or target mismatch")
    for key in CONTEXT_FIELDS:
        check(isinstance(context[key], str) and context[key] and "\n" not in context[key] and "\r" not in context[key], "Missing exact native context: " + key)
    for key in ["lockHash", "sourceHash", "buildPolicyHash"]:
        exact_sha(context[key], 64)


def create_manifest(root, context, origin, products):
    """Called only after a successful producer with measured exact build context."""
    validate_context(context)
    exact_sha(origin["originBuildSHA"]); exact_sha(origin["originTree"])
    check(origin["producerStatus"] == "success", "Failed producer cannot certify bytes")
    check(0 < len(products) <= MAX_FILES, "Incomplete producer product set")
    real_root = Path(root).resolve(strict=True)
    records, total = [], 0
    for row in products:
        member = safe_member(row["file"])
        current = real_root
        for part in PurePosixPath(member).parts:
            current = current / part
            check(not current.is_symlink() and not current.is_junction(), "Producer product redirects")
            check(current.resolve(strict=True).is_relative_to(real_root), "Producer product component escaped root")
        resolved = current.resolve(strict=True)
        check(resolved.is_relative_to(real_root) and resolved.is_file(), "Producer product escaped root")
        total += resolved.stat().st_size
        check(total <= MAX_ARCHIVE, "Producer bytes exceed bound")
        data = resolved.read_bytes()
        records.append({"name": row["name"], "file": member, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()})
    check(len({row["file"] for row in records}) == len(records), "Duplicate producer member")
    validate_member_set([row["file"] for row in records], context["abi"])
    check(len({row["name"] for row in records}) == len(records), "Duplicate producer package")
    return {"schemaVersion": SCHEMA, "kind": "native-build-bytes-not-acceptance", "origin": origin, "context": context, "inputHash": digest(context), "records": records}


def inspect_reuse(archive_bytes, github, expected):
    """Returns original verified bytes plus new consumer binding; never new origin.

    github must come from the allowed GitHub API's run/attempt/job/artifact data.
    Supplying a local receipt as github evidence is not an authorized live path.
    """
    validate_context(expected["context"])
    for key in ["consumerSHA", "testedSHA", "testedTree", "mergeTree"]:
        exact_sha(expected[key])
    exact_sha(expected["planHash"], 64)
    run, job, artifact = github["run"], github["job"], github["artifact"]
    check(run["repositoryId"] == expected["repositoryId"], "Wrong producer repository")
    check(run["status"] == "completed" and job["status"] == "completed" and job["conclusion"] == "success", "Producer job was not successful")
    check(job["runId"] == run["id"] and job["runAttempt"] == run["attempt"], "Producer job attempt mismatch")
    check(job["name"] == expected["producerJobName"] and run["path"] == expected["workflowPath"], "Unexpected producer workflow/job")
    check(run["headRepositoryId"] == expected["repositoryId"] and run["event"] in ["pull_request", "push", "workflow_dispatch"], "Untrusted producer event or fork")
    check(artifact["runId"] == run["id"] and artifact["runAttempt"] == run["attempt"] and artifact["expired"] is False, "Expired or wrong producer artifact")
    check(isinstance(artifact["id"], int) and artifact["id"] > 0, "Immutable artifact ID missing")
    check(len(archive_bytes) <= MAX_ARCHIVE and len(archive_bytes) == artifact["size"], "Artifact archive size differs")
    archive_hash = hashlib.sha256(archive_bytes).hexdigest()
    check(artifact["digest"] == "sha256:" + archive_hash, "GitHub artifact digest differs")
    with zipfile.ZipFile(io.BytesIO(archive_bytes)) as archive:
        entries = archive.infolist()
        check(0 < len(entries) <= MAX_FILES + 1, "Artifact file count outside bound")
        check(len({entry.filename for entry in entries}) == len(entries), "Duplicate ZIP member")
        validate_member_set([entry.filename for entry in entries], expected["context"]["abi"])
        check(sum(entry.file_size for entry in entries) <= MAX_ARCHIVE, "Expanded artifact exceeds bound")
        for entry in entries:
            safe_member(entry.filename)
            check(not entry.is_dir() and stat.S_IFMT(entry.external_attr >> 16) in [0, stat.S_IFREG], "Artifact directory, symlink or special file is forbidden")
        check(archive.getinfo("native-proof.json").file_size <= 256 * 1024, "Native proof too large")
        proof_bytes = archive.read("native-proof.json")
        check(len(proof_bytes) <= 256 * 1024, "Native proof too large")
        proof = json.loads(proof_bytes)
        check(proof["schemaVersion"] == SCHEMA and proof["kind"] == "native-build-bytes-not-acceptance", "Legacy or unknown native proof")
        check(proof["context"] == expected["context"] and proof["inputHash"] == digest(expected["context"]), "Native input or environment differs")
        origin = proof["origin"]
        # run.head_sha may identify a PR head, not the synthetic checkout commit.
        # The adapter must independently read that commit and hash its complete
        # input closure and reviewed producer code. Artifact claims are not this proof.
        source = github["sourceProof"]
        check(origin["originBuildSHA"] == source["commitSHA"] and origin["originTree"] == source["tree"], "Producer source identity differs")
        check(source["sourceHash"] == expected["context"]["sourceHash"] and source["buildPolicyHash"] == expected["context"]["buildPolicyHash"], "Producer source inputs or code differ")
        check(source["workflowHash"] == expected["workflowHash"], "Producer workflow was not reviewed")
        if origin["originBuildSHA"] != run["headSHA"]:
            check(run["event"] == "pull_request" and {run["headSHA"], run["baseSHA"]}.issubset(set(source["parents"])), "Checkout is not the run's verified test merge")
        check(origin["runId"] == run["id"] and origin["runAttempt"] == run["attempt"], "Producer run identity differs")
        if "jobId" in origin:
            check(origin["jobId"] == job["id"], "Producer job identity differs")
        else:
            check(origin["jobKey"] == expected["producerJobKey"] and origin["jobName"] == expected["producerJobName"], "Producer logical job identity differs")
        check(origin["workflowPath"] == run["path"] and origin["producerStatus"] == "success", "Producer policy differs")
        check(proof["records"] and len(proof["records"]) == len(expected["products"]), "Incomplete native cohort")
        wanted = {(row["name"], row["file"]) for row in expected["products"]}
        check(len(wanted) == len(expected["products"]) and {(row["name"], row["file"]) for row in proof["records"]} == wanted, "Wrong or duplicate native products")
        check({entry.filename for entry in entries} == {"native-proof.json"} | {row["file"] for row in proof["records"]}, "Unexpected artifact member")
        products = {}
        for row in proof["records"]:
            body = archive.read(safe_member(row["file"]))
            check(len(body) == row["bytes"] and hashlib.sha256(body).hexdigest() == row["sha256"], "Native artifact bytes differ")
            products[row["file"]] = body
    return {"action": "verified-build-byte-reuse", "origin": origin, "verifiedProducerJobId": job["id"], "consumerSHA": expected["consumerSHA"], "testedSHA": expected["testedSHA"], "testedTree": expected["testedTree"], "mergeTree": expected["mergeTree"], "planHash": expected["planHash"], "artifactId": artifact["id"], "artifactDigest": artifact["digest"], "acceptanceInherited": False, "products": products}


def resolve_reuse(expected, candidates, load, max_candidates=3):
    """Bounded adapter. Every error, including 403, missing proof or expiry builds."""
    check(isinstance(max_candidates, int) and 0 < max_candidates <= 3, "Candidate scan must be bounded")
    reasons = []
    for candidate in candidates[:max_candidates]:
        try:
            archive_bytes, github = load(candidate)
            return inspect_reuse(archive_bytes, github, expected)
        except Exception as error:
            reasons.append(str(error)[:500])
            if isinstance(error, PermissionError):
                break
    return {"action": "build", "acceptanceInherited": False, "reasons": reasons or ["No complete trusted producer proof; legacy artifacts are not reusable"]}
