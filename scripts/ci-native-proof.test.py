# cspell:words importlib
"""Synthetic byte-proof contracts; no compiler, container or network access."""
import copy
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
import zipfile

spec = importlib.util.spec_from_file_location("native_proof", Path(__file__).with_name("ci-native-proof.py"))
proof = importlib.util.module_from_spec(spec); spec.loader.exec_module(proof)


def fixture():
    context = {name: "measured" for name in proof.CONTEXT_FIELDS}
    context.update({"profile": "ci", "abi": "linux-x64-gnu", "target": "x86_64-unknown-linux-gnu"})
    for name in ["lockHash", "sourceHash", "buildPolicyHash"]:
        context[name] = "a" * 64
    origin = {"originBuildSHA": "a" * 40, "originTree": "b" * 40, "runId": 10, "runAttempt": 2, "jobId": 20, "workflowPath": ".github/workflows/ci.yaml", "producerStatus": "success"}
    body = b"synthetic native byte contract, not an executable"
    record = {"name": "@farmfe/core-linux-x64-gnu", "file": "payload/farm.node", "sha256": hashlib.sha256(body).hexdigest(), "bytes": len(body)}
    manifest = {"schemaVersion": 1, "kind": "native-build-bytes-not-acceptance", "context": context, "inputHash": proof.digest(context), "origin": origin, "records": [record]}
    expected = {"repositoryId": 1, "context": context, "consumerSHA": "c" * 40, "testedSHA": "d" * 40, "testedTree": "e" * 40, "mergeTree": "f" * 40, "planHash": "1" * 64, "workflowHash": "2" * 64, "producerJobName": "Native Linux", "workflowPath": origin["workflowPath"], "products": [{"name": record["name"], "file": record["file"]}]}
    github = {"run": {"id": 10, "attempt": 2, "repositoryId": 1, "headRepositoryId": 1, "event": "pull_request", "path": origin["workflowPath"], "headSHA": origin["originBuildSHA"], "tree": origin["originTree"], "status": "completed", "conclusion": "failure"},
              "job": {"id": 20, "runId": 10, "runAttempt": 2, "name": "Native Linux", "status": "completed", "conclusion": "success"},
              "artifact": {"id": 30, "runId": 10, "runAttempt": 2, "expired": False},
              "sourceProof": {"commitSHA": origin["originBuildSHA"], "tree": origin["originTree"], "sourceHash": context["sourceHash"], "buildPolicyHash": context["buildPolicyHash"], "workflowHash": expected["workflowHash"], "parents": []}}
    return manifest, expected, github, body


def bundle(manifest, github, body, extra=None):
    output = io.BytesIO()
    with zipfile.ZipFile(output, "w") as archive:
        archive.writestr("native-proof.json", json.dumps(manifest))
        archive.writestr("payload/farm.node", body)
        if extra:
            archive.writestr(extra, b"unexpected")
    data = output.getvalue()
    github["artifact"].update({"size": len(data), "digest": "sha256:" + hashlib.sha256(data).hexdigest()})
    return data


class NativeProofContracts(unittest.TestCase):
    def test_producer_manifest_records_exact_bytes_and_rejects_traversal(self):
        manifest, _, _, body = fixture()
        with tempfile.TemporaryDirectory(prefix="farm-native-proof-") as directory:
            root = Path(directory); (root / "payload").mkdir(); (root / "payload/farm.node").write_bytes(body)
            products = [{"name": "@farmfe/core-linux-x64-gnu", "file": "payload/farm.node"}]
            created = proof.create_manifest(root, manifest["context"], manifest["origin"], products)
            self.assertEqual(created["records"], manifest["records"])
            with self.assertRaises(ValueError):
                proof.create_manifest(root, manifest["context"], manifest["origin"], [{"name": "escape", "file": "../escape"}])

    def test_successful_producer_in_failed_aggregate_reuses_only_bytes(self):
        manifest, expected, github, body = fixture()
        result = proof.inspect_reuse(bundle(manifest, github, body), github, expected)
        self.assertEqual(result["origin"]["originBuildSHA"], "a" * 40)
        self.assertEqual(result["consumerSHA"], "c" * 40)
        self.assertFalse(result["acceptanceInherited"])
        self.assertEqual(result["products"]["payload/farm.node"], body)

    def test_cancelled_failed_partial_expired_or_wrong_attempt_rebuilds(self):
        for section, key, value in [("job", "conclusion", "failure"), ("job", "conclusion", "cancelled"), ("job", "status", "in_progress"), ("artifact", "expired", True), ("artifact", "runAttempt", 1), ("run", "headRepositoryId", 99)]:
            manifest, expected, github, body = fixture(); github[section][key] = value
            archive = bundle(manifest, github, body)
            self.assertEqual(proof.resolve_reuse(expected, [30], lambda _: (archive, github))["action"], "build")

    def test_profile_abi_input_and_byte_mismatch_rebuild(self):
        for key, value in [("profile", "release"), ("abi", "darwin-arm64"), ("sourceHash", "b" * 64), ("buildPolicyHash", "c" * 64)]:
            manifest, expected, github, body = fixture()
            expected = copy.deepcopy(expected); expected["context"][key] = value
            archive = bundle(manifest, github, body)
            self.assertEqual(proof.resolve_reuse(expected, [30], lambda _: (archive, github))["action"], "build")
        manifest, expected, github, body = fixture()
        archive = bundle(manifest, github, body + b"changed")
        self.assertEqual(proof.resolve_reuse(expected, [30], lambda _: (archive, github))["action"], "build")

    def test_unknown_or_escaping_archive_member_rebuilds(self):
        for member in ["../escape", "extra-file", "C:/escape", "dir\\escape"]:
            manifest, expected, github, body = fixture(); archive = bundle(manifest, github, body, member)
            self.assertEqual(proof.resolve_reuse(expected, [30], lambda _: (archive, github))["action"], "build")

    def test_members_reject_noncanonical_windows_reserved_and_filesystem_collisions(self):
        for member in ["payload//farm.node", "./payload/farm.node", "payload/./farm.node", "payload/", "payload/CON.node", "payload/lpt1", "payload/COM¹.bin", "payload/file. ", "payload/file.", "payload/line\nname", "payload/control\u202ename", "payload/cafe\u0301.node"]:
            with self.subTest(member=member), self.assertRaises(ValueError):
                proof.safe_member(member)
        for abi in ["win32-x64-msvc", "darwin-arm64"]:
            with self.assertRaises(ValueError):
                proof.validate_member_set(["payload/farm.node", "payload/FARM.node"], abi)
        proof.validate_member_set(["payload/farm.node", "payload/FARM.node"], "linux-x64-gnu")

    def test_missing_legacy_proof_and_permission_error_rebuild(self):
        _, expected, github, _ = fixture()
        self.assertEqual(proof.resolve_reuse(expected, [], lambda _: None)["action"], "build")
        output = io.BytesIO()
        with zipfile.ZipFile(output, "w") as archive:
            archive.writestr("payload/farm.node", b"legacy product without provenance")
        data = output.getvalue()
        github["artifact"].update({"size": len(data), "digest": "sha256:" + hashlib.sha256(data).hexdigest()})
        self.assertEqual(proof.resolve_reuse(expected, [30], lambda _: (data, github))["action"], "build")
        calls = []
        def denied(candidate):
            calls.append(candidate); raise PermissionError("403: artifact read unavailable")
        result = proof.resolve_reuse(expected, list(range(100)), denied)
        self.assertEqual(result["action"], "build"); self.assertEqual(len(calls), 1)
        self.assertFalse(result["acceptanceInherited"])

    def test_missing_product_or_wrong_github_digest_rebuilds(self):
        manifest, expected, github, body = fixture(); archive = bundle(manifest, github, body)
        expected["products"].append({"name": "missing", "file": "payload/missing.farm"})
        self.assertEqual(proof.resolve_reuse(expected, [30], lambda _: (archive, github))["action"], "build")
        manifest, expected, github, body = fixture(); archive = bundle(manifest, github, body)
        github["artifact"]["digest"] = "sha256:" + "0" * 64
        self.assertEqual(proof.resolve_reuse(expected, [30], lambda _: (archive, github))["action"], "build")

    def test_actual_test_merge_is_distinct_from_pr_head_and_bound_to_both_parents(self):
        manifest, expected, github, body = fixture()
        github["run"].update({"headSHA": "4" * 40, "baseSHA": "5" * 40})
        github["sourceProof"]["parents"] = ["4" * 40, "5" * 40]
        archive = bundle(manifest, github, body)
        self.assertEqual(proof.inspect_reuse(archive, github, expected)["origin"]["originBuildSHA"], "a" * 40)
        github["sourceProof"]["parents"] = ["4" * 40]
        self.assertEqual(proof.resolve_reuse(expected, [30], lambda _: (archive, github))["action"], "build")


if __name__ == "__main__":
    unittest.main()
