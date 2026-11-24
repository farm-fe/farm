"""Policy for a future signed, retain-only Docs pair receipt.

This module only evaluates policy at a future trusted verifier boundary.
It cannot sign, fetch, spawn a tool or grant publication authority. Expected
policy and authentic verification results must come from separate reviewed
trust boundaries, never from the untrusted receipt or persisted verdict JSON.
"""
import base64
import binascii
import hashlib
import json
import re

REPOSITORY = "farm-fe/farm"
REPOSITORY_ID = 507542208
REF = "refs/heads/main"
ISSUER = "https://token.actions.githubusercontent.com"
PREDICATE = "https://farmfe.org/attestations/docs-pair-retained/v1"
SUBJECT = "docs-pair-retained.json"
PAIR = ("@farmfe/docs-theme-default", "@farmfe/docs")
KIND = "docs-public-retain-only-v1"
PACKAGE_FIELDS = {"name", "version", "archiveSha256", "archiveBytes", "integrity", "resolved", "metadataSha256"}
POLICY_FIELDS = {"repositoryId", "signerWorkflow", "callerWorkflow", "signerSHA",
                 "originalSourceSHA", "originalSourceTree", "runId", "attempt", "frozenArtifact", "originalArchives"}
ARTIFACT_FIELDS = {"runId", "attempt", "artifactId", "artifactSha256"}
ARCHIVE_FIELDS = {"archiveSha256", "archiveBytes", "integrity"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def hex_digest(value, count):
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{%d}" % count, value) is not None


def strict_json(body):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, "Duplicate retention receipt key")
            result[key] = value
        return result

    def bad_constant(_):
        raise ValueError("Non-finite retention receipt value")

    return json.loads(body.decode("utf-8"), object_pairs_hook=pairs, parse_constant=bad_constant)


def valid_sri(value):
    if not isinstance(value, str) or not value.startswith("sha512-"):
        return False
    try:
        raw = base64.b64decode(value[7:], validate=True)
    except (ValueError, binascii.Error):
        return False
    return len(raw) == 64 and base64.b64encode(raw).decode("ascii") == value[7:]


def validate_receipt(body, expected):
    require(type(body) is bytes and 0 < len(body) <= 32768, "Invalid signed retention receipt bytes")
    require(isinstance(expected, dict) and set(expected) == POLICY_FIELDS, "Independent retention signing policy required")
    for key in ("repositoryId", "runId", "attempt"):
        require(type(expected[key]) is int and expected[key] > 0, "Invalid retention signing run identity")
    for key in ("signerSHA", "originalSourceSHA", "originalSourceTree"):
        require(hex_digest(expected[key], 40), "Exact retention signing/source object required")
    for key in ("signerWorkflow", "callerWorkflow"):
        require(isinstance(expected[key], str) and re.fullmatch(r"\.github/workflows/[a-z0-9.-]+\.ya?ml", expected[key]) is not None,
                "Exact reviewed retention workflow required")
    require(expected["repositoryId"] == REPOSITORY_ID, "Canonical Farm repository required")
    artifact = expected["frozenArtifact"]
    require(type(artifact) is dict and set(artifact) == ARTIFACT_FIELDS
            and type(artifact["runId"]) is int and artifact["runId"] > 0 and artifact["runId"] != expected["runId"]
            and type(artifact["attempt"]) is int and artifact["attempt"] == 1
            and type(artifact["artifactId"]) is int and artifact["artifactId"] > 0
            and hex_digest(artifact["artifactSha256"], 64), "Independently verified original first-attempt artifact required")
    archives = expected["originalArchives"]
    require(type(archives) is dict and set(archives) == set(PAIR)
            and all(type(row) is dict and set(row) == ARCHIVE_FIELDS for row in archives.values()),
            "Both original package archives required")
    receipt = strict_json(body)
    require(isinstance(receipt, dict) and set(receipt) == {"schemaVersion", "kind", "sourceSHA", "sourceTree", "packages"}
            and type(receipt["schemaVersion"]) is int and receipt["schemaVersion"] == 1 and receipt["kind"] == KIND,
            "Invalid retained-only package proof shape")
    require(receipt["sourceSHA"] == expected["originalSourceSHA"] and receipt["sourceTree"] == expected["originalSourceTree"],
            "Retained proof differs from exact original Docs source/tree")
    rows = receipt["packages"]
    require(isinstance(rows, list) and len(rows) == len(PAIR), "Retention requires the complete Docs pair")
    for row, name in zip(rows, PAIR):
        slug = name.split("/")[1]
        urls = [f"https://registry.npmjs.org/{candidate}/-/{slug}-2.0.0.tgz" for candidate in (name, name.replace("/", "%2F"))]
        require(isinstance(row, dict) and set(row) == PACKAGE_FIELDS and row["name"] == name
                and row["version"] == "2.0.0" and row["resolved"] in urls
                and type(row["archiveBytes"]) is int and 0 < row["archiveBytes"] <= 100 * 1024 * 1024
                and hex_digest(row["archiveSha256"], 64) and hex_digest(row["metadataSha256"], 64)
                and valid_sri(row["integrity"]), "Invalid exact retained package row")
        require(all(type(row[k]) is type(archives[name][k]) and row[k] == archives[name][k] for k in ARCHIVE_FIELDS),
                "Retained package differs from independently verified original archive")
    return receipt


def cert_uri(workflow):
    return f"https://github.com/{REPOSITORY}/{workflow}@{REF}"


def verifier_args(expected):
    """Additional argv to the SHA-pinned gh attestation verify subcommand."""
    return ["--repo", REPOSITORY, "--cert-identity", cert_uri(expected["signerWorkflow"]),
            "--signer-digest", expected["signerSHA"], "--source-digest", expected["signerSHA"],
            "--source-ref", REF, "--predicate-type", PREDICATE, "--cert-oidc-issuer", ISSUER,
            "--deny-self-hosted-runners", "--format", "json"]


def verify_retained_attestation(body, bundle, *, expected, verify):
    """verify(body, bundle, flags) is the trusted current CLI subprocess seam.

    It must run reviewed pinned gh on the bytes under SUBJECT with the bundle,
    require success, and return fresh JSON output. This adapter deliberately
    does not accept a verified-result JSON path or bundled certificate claims.
    """
    require(type(expected) is dict, "Independent retention signing policy required")
    receipt = validate_receipt(body, expected)
    expected = {**expected, "frozenArtifact": dict(expected["frozenArtifact"])}
    require(type(bundle) is bytes and 0 < len(bundle) <= 16 * 1024 * 1024, "Invalid retention attestation bundle")
    result = verify(body, bundle, verifier_args(expected))
    require(isinstance(result, list) and len(result) == 1 and isinstance(result[0], dict), "Missing or ambiguous genuine verification")
    verified = result[0].get("verificationResult", {})
    require(isinstance(verified, dict) and isinstance(verified.get("signature"), dict), "Missing verified signature")
    certificate = verified["signature"].get("certificate")
    repo = f"https://github.com/{REPOSITORY}"
    signer = cert_uri(expected["signerWorkflow"])
    identities = {"issuer": ISSUER, "subjectAlternativeName": signer, "buildSignerURI": signer,
                  "buildSignerDigest": expected["signerSHA"], "runnerEnvironment": "github-hosted",
                  "sourceRepositoryURI": repo, "sourceRepositoryIdentifier": str(expected["repositoryId"]),
                  "sourceRepositoryDigest": expected["signerSHA"], "sourceRepositoryRef": REF,
                  "buildConfigURI": cert_uri(expected["callerWorkflow"]), "buildConfigDigest": expected["signerSHA"],
                  "buildTrigger": "push",
                  "runInvocationURI": f"{repo}/actions/runs/{expected['runId']}/attempts/{expected['attempt']}"}
    require(isinstance(certificate, dict) and all(certificate.get(key) == value for key, value in identities.items()),
            "Protected main signer certificate identity differs")
    statement = verified.get("statement", {})
    require(isinstance(statement, dict) and statement.get("_type") == "https://in-toto.io/Statement/v1" and statement.get("predicateType") == PREDICATE
            and statement.get("predicate") == {"schemaVersion": 1, "kind": KIND, "frozenArtifact": expected["frozenArtifact"]}
            and statement.get("subject") == [{"name": SUBJECT, "digest": {"sha256": hashlib.sha256(body).hexdigest()}}],
            "Exact retained proof attestation subject/predicate differs")
    return receipt
