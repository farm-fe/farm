# cspell:words importlib
"""API-shaped fixtures only; no local or remote GitHub requests."""
import copy
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock

spec = importlib.util.spec_from_file_location("evidence", Path(__file__).with_name("ci-native-evidence.py"))
evidence = importlib.util.module_from_spec(spec); spec.loader.exec_module(evidence)


def fixture():
    expected = {"repositoryId": 1, "trustedPolicySHA": "a" * 40, "workflowPath": ".github/workflows/ci.yaml", "producerJobName": "Native Linux", "context": {"abi": "linux-x64-gnu"}}
    candidate = {"runId": 10, "attempt": 2, "artifactId": 30, "originBuildSHA": "b" * 40}
    run = {"id": 10, "run_attempt": 2, "repository": {"id": 1}, "head_repository": {"id": 1}, "event": "pull_request", "path": ".github/workflows/ci.yaml", "head_sha": "c" * 40, "status": "completed", "pull_requests": [{"base": {"sha": "d" * 40}}]}
    job = {"id": 20, "run_id": 10, "name": "Native Linux", "status": "completed", "conclusion": "success", "started_at": "2026-10-30T10:00:00Z", "completed_at": "2026-10-30T10:02:00Z"}
    artifact = {"id": 30, "name": "native-proof-" + "b" * 40 + "-linux-x64-gnu-2", "workflow_run": {"id": 10}, "expired": False, "created_at": "2026-10-30T10:01:00Z", "size_in_bytes": 7, "digest": "sha256:" + "e" * 64}
    routes = {"": {"id": 1, "default_branch": "main"}, "/branches/main": {"commit": {"sha": "a" * 40}},
              "/actions/runs/10/attempts/2": run, "/actions/runs/10/attempts/2/jobs?per_page=100&page=1": {"jobs": [job]}, "/actions/artifacts/30": artifact}
    class Transport:
        def __init__(self):
            self.routes = routes; self.calls = []; self.downloads = []
        def get(self, route):
            self.calls.append(route); return self.routes[route]
        def archive(self, artifact_id):
            self.downloads.append(artifact_id); return b"archive"
    def source_reader(commit, _):
        return {"commitSHA": commit, "tree": "f" * 40, "sourceHash": "1" * 64, "buildPolicyHash": "2" * 64, "workflowHash": "3" * 64, "parents": []}
    return expected, candidate, Transport(), source_reader


class EvidenceContracts(unittest.TestCase):
    def test_source_ledger_explicitly_binds_the_platform_orchestration(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); products = root / "products.json"; output = root / "source.json"
            products.write_text(json.dumps({"records": [{"owner": "@farmfe/core"}], "abi": "linux-x64-gnu", "profile": "ci"}), encoding="utf-8")
            environment = {"GITHUB_SHA": "a" * 40, "GITHUB_RUN_ID": "10", "GITHUB_RUN_ATTEMPT": "2", "GITHUB_JOB": "build"}
            with mock.patch.dict(evidence.os.environ, environment, clear=True), mock.patch.object(evidence, "committed_source", return_value={"buildPolicyHash": "b" * 64}) as committed:
                evidence.capture_source(products, output)
            commit, expected = committed.call_args.args
            self.assertEqual(commit, "a" * 40)
            for file in ["ci.yaml", "ci-primary-platform.yaml", "rust-build-user.yaml", "rust-build-release.yaml"]:
                self.assertIn(".github/workflows/" + file, expected["policyFiles"])
            record = json.loads(output.read_text(encoding="utf-8"))
            self.assertEqual(record["policyFiles"], expected["policyFiles"])
            self.assertEqual(record["source"]["buildPolicyHash"], "b" * 64)
            self.assertFalse(record["reuseAuthorized"])

    def test_identity_comes_from_api_not_artifact_claims(self):
        expected, candidate, transport, reader = fixture()
        data, github = evidence.GitHubEvidenceAdapter(transport, reader).load(candidate, expected)
        self.assertEqual(data, b"archive"); self.assertEqual(github["job"]["id"], 20)
        self.assertEqual(github["run"]["headSHA"], "c" * 40)
        self.assertEqual(github["sourceProof"]["commitSHA"], "b" * 40)
        self.assertEqual(transport.downloads, [30])

    def test_failed_or_ambiguous_job_never_downloads(self):
        for ambiguous in [False, True]:
            expected, candidate, transport, reader = fixture()
            jobs = transport.routes["/actions/runs/10/attempts/2/jobs?per_page=100&page=1"]["jobs"]
            if ambiguous:
                jobs.append(copy.deepcopy(jobs[0]))
            else:
                jobs[0]["conclusion"] = "cancelled"
            with self.assertRaises(ValueError):
                evidence.GitHubEvidenceAdapter(transport, reader).load(candidate, expected)
            self.assertEqual(transport.downloads, [])

    def test_wrong_attempt_artifact_and_moved_policy_never_download(self):
        for change in ["artifact", "policy"]:
            expected, candidate, transport, reader = fixture()
            if change == "artifact":
                transport.routes["/actions/artifacts/30"]["created_at"] = "2026-10-29T10:01:00Z"
            else:
                transport.routes["/branches/main"]["commit"]["sha"] = "0" * 40
            with self.assertRaises(ValueError):
                evidence.GitHubEvidenceAdapter(transport, reader).load(candidate, expected)
            self.assertEqual(transport.downloads, [])

    def test_live_transport_cannot_be_used_for_local_denial_retry(self):
        with self.assertRaises(PermissionError):
            evidence.GitHubReadOnly("farm-fe/farm", "fixture-not-a-token", environment={})

    def test_shadow_archive_transport_is_explicitly_disabled(self):
        client = evidence.GitHubReadOnly("farm-fe/farm", "fixture-not-a-token", environment={"GITHUB_ACTIONS": "true"})
        with self.assertRaises(PermissionError):
            client.archive(30)

    def test_storage_redirect_is_bounded_and_never_forwards_authorization(self):
        client = evidence.GitHubReadOnly("farm-fe/farm", "fixture-not-a-token", environment={"GITHUB_ACTIONS": "true"}, allow_archive=True)
        calls = []
        class Opener:
            def open(self, request, timeout):
                calls.append(request); self.timeout = timeout
                if len(calls) == 1:
                    raise evidence.urllib.error.HTTPError(request.full_url, 302, "Found", {"Location": "https://storage.blob.core.windows.net/artifact?fixture=1"}, None)
                return io.BytesIO(b"archive")
        client.opener = Opener()
        self.assertEqual(client.archive(30), b"archive")
        self.assertIsNotNone(calls[0].get_header("Authorization"))
        self.assertIsNone(calls[1].get_header("Authorization"))
        for target in ["http://storage.blob.core.windows.net/artifact", "https://evil.invalid/artifact", "https://storage.blob.core.windows.net.evil.invalid/artifact", "https://user:password@storage.blob.core.windows.net/artifact"]:
            with self.assertRaises(ValueError):
                evidence.artifact_redirect_url(target)


if __name__ == "__main__":
    unittest.main()
