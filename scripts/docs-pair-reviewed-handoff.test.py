"""Isolated read-only role and archive checks; no GitHub or npm transport."""
# cspell:words importlib zipfile
import base64
from copy import deepcopy
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import unittest
import zipfile


spec = importlib.util.spec_from_file_location(
    "reviewed_docs_handoff", Path(__file__).with_name("docs-pair-reviewed-handoff.py")
)
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)


class Boundary(unittest.TestCase):
    def setUp(self):
        sha, tree = "a"*40, "b"*40
        self.bodies = {name: bytes([17+i])*3 for i, name in enumerate(m.TARS)}
        self.receipt = {"schemaVersion": 1, "kind": "docs-frozen-pair-v1", "sourceSHA": sha,
                        "sourceTree": tree, "packages": []}
        for name, body in self.bodies.items():
            self.receipt["packages"].append({"name": name, "version": "2.0.0", "archiveBytes": len(body),
                    "archiveSha256": hashlib.sha256(body).hexdigest(),
                    "integrity": "sha512-" + base64.b64encode(hashlib.sha512(body).digest()).decode("ascii")})
        self.selection = {"sourceSHA": sha, "sourceTree": tree, "producerRun": 10, "writerRun": 20,
                          "artifactId": 30, "artifactSha256": "", "tag": "farm-docs-bootstrap"}
        self.policy = {"producerWorkflowId": m.PRODUCER_ID, "writerWorkflowId": 21, "environmentId": 40,
                       "environmentName": m.ENVIRONMENT_NAME, "trustedReviewerIds": frozenset({50})}
        repo = {"id": m.REPOSITORY_ID, "full_name": "farm-fe/farm"}
        base = {"head_sha": sha, "head_branch": "main", "run_attempt": 1, "event": "workflow_dispatch",
                "actor": {"id": 22}, "repository": repo, "head_repository": deepcopy(repo)}
        p = {**base, "id": 10, "workflow_id": m.PRODUCER_ID, "path": m.PRODUCER, "status": "completed", "conclusion": "success"}
        w = {**base, "id": 20, "workflow_id": 21, "path": m.WRITER, "status": "in_progress", "conclusion": None}
        job = {"name": m.JOB, "id": 19, "run_id": 10, "head_sha": sha, "status": "completed",
               "conclusion": "success", "steps": [{"name": m.UPLOAD, "status": "completed", "conclusion": "success"}]}
        self.origins = {"/branches/main": {"name": "main", "protected": True, "commit": {"sha": sha}},
            f"/git/commits/{sha}": {"sha": sha, "tree": {"sha": tree}},
            "/actions/runs/10": p, "/actions/runs/10/attempts/1": deepcopy(p),
            "/actions/runs/20": w, "/actions/runs/20/attempts/1": deepcopy(w),
            "/actions/runs/10/attempts/1/jobs": {"total_count": 1, "jobs": [job]},
            "/actions/artifacts/30": {"id": 30, "name": f"farm-docs-pair-{sha}-1", "expired": False,
                "workflow_run": {"id": 10, "repository_id": m.REPOSITORY_ID, "head_repository_id": m.REPOSITORY_ID,
                                 "head_branch": "main", "head_sha": sha}}}
        self.history = "/actions/runs/20/approvals"
        self.calls = []
        self.freeze()

    def freeze(self, *, extra=False, archive_changes=None):
        bodies = self.bodies if archive_changes is None else {**self.bodies, **archive_changes}
        sink = io.BytesIO()
        with zipfile.ZipFile(sink, "w") as z:
            z.writestr(m.RECEIPT, json.dumps(self.receipt))
            for name, filename in m.TARS.items():
                z.writestr(filename, bodies[name])
            if extra:
                z.writestr("extra.txt", "not reviewed")
        self.zip = sink.getvalue()
        self.selection["artifactSha256"] = hashlib.sha256(self.zip).hexdigest()
        self.origins["/actions/artifacts/30"]["digest"] = "sha256:" + self.selection["artifactSha256"]
        permit = {**self.selection, "attempt": 1, "environmentId": 40,
                  "archives": {v["name"]: v["archiveSha256"] for v in self.receipt["packages"]}}
        self.origins[self.history] = [{"state": "approved", "comment": m.owner.approval_comment(permit),
                         "user": {"id": 50}, "environments": [{"id": 40, "name": m.ENVIRONMENT_NAME}]}]

    def run_contract(self, getter=None):
        def get(route):
            self.calls.append(route)
            self.assertIn(route, self.origins)
            return self.origins[route]
        def download(artifact, limit):
            self.assertEqual((artifact, limit), (30, m.MAX_ZIP))
            self.calls.append("download-frozen-30")
            return self.zip
        return m.read_reviewed_handoff(self.selection, self.policy, get=get if getter is None else getter, download=download)

    def test_valid_split_role_order_and_exact_result(self):
        result = self.run_contract()
        self.assertEqual(result, {"producerJobId": 19, "artifactId": 30, "receipt": self.receipt, "archives": self.bodies})
        self.assertEqual(self.calls[-4:], ["download-frozen-30", self.history, "/branches/main",
                                         f"/git/commits/{self.selection['sourceSHA']}"])

    def test_wrong_role_path_workflow_id_run_id_repo_and_replay_precede_artifact(self):
        for route, key, value in [
            ("/actions/runs/10", "path", m.WRITER),
            ("/actions/runs/20", "path", m.PRODUCER),
            ("/actions/runs/20", "workflow_id", 11),
            ("/actions/runs/10", "id", 20),
            ("/actions/runs/20", "run_attempt", 2),
            ("/actions/runs/10", "run_attempt", 2),
            ("/actions/runs/20/attempts/1", "id", 19),
            ("/actions/runs/20/attempts/1", "conclusion", "success"),
            ("/actions/runs/10", "event", "pull_request"),
            ("/actions/runs/10", "head_repository", {"id": 1, "full_name": "farm-fe/farm"}),
            ("/actions/runs/20/attempts/1", "actor", {"id": 20})]:
            old = self.origins[route][key]
            self.origins[route][key] = value
            with self.subTest(route=route, key=key), self.assertRaises(ValueError):
                self.run_contract()
            self.origins[route][key] = old
        self.assertNotIn("/actions/artifacts/30", self.calls)
        self.assertNotIn(self.history, self.calls)

    def test_dispatchers_cannot_alias_canonical_roles(self):
        for obj, key, value in [
            (self.policy, "writerWorkflowId", m.PRODUCER_ID), (self.selection, "writerRun", 10),
            (self.selection, "producerRun", True), (self.selection, "tag", "latest"),
            (self.policy, "environmentName", "Production"), (self.policy, "producerWorkflowId", 999),
            (self.policy, "writerWorkflowId", None), (self.policy, "environmentId", None)]:
            old = obj[key]
            obj[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.run_contract()
            obj[key] = old
        self.assertEqual(self.calls, [])

    def test_incomplete_jobs_failed_upload_and_artifact_origin_fail_before_download(self):
        page = self.origins["/actions/runs/10/attempts/1/jobs"]
        job = page["jobs"][0]
        step = job["steps"][0]
        artifact = self.origins["/actions/artifacts/30"]
        for obj, key, value in [
            (page, "total_count", 2), (page, "jobs", [job, job]),
            (step, "conclusion", "failure"), (artifact["workflow_run"], "id", 20),
            (artifact["workflow_run"], "head_repository_id", 999), (artifact, "expired", True)]:
            old = obj[key]
            obj[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.run_contract()
            obj[key] = old
        self.assertNotIn("download-frozen-30", self.calls)
        self.assertNotIn(self.history, self.calls)

    def test_mutated_zip_resealed_repack_and_extra_entry_never_reach_approval(self):
        original = self.zip
        self.zip = original + b"non-original"
        with self.assertRaisesRegex(ValueError, "ZIP bytes differ"):
            self.run_contract()
        self.freeze(archive_changes={m.owner.PACKAGE_ORDER[0]: b"bad"})
        with self.assertRaisesRegex(ValueError, "original package bytes"):
            self.run_contract()
        self.freeze(extra=True)
        with self.assertRaisesRegex(ValueError, "exact original files"):
            self.run_contract()
        self.assertNotIn(self.history, self.calls)

    def test_missing_old_replayed_self_approved_wrong_environment_and_ambiguous_denied(self):
        current = deepcopy(self.origins[self.history][0])
        stale = {**self.selection, "writerRun": 21, "attempt": 1, "environmentId": 40,
                 "archives": {v["name"]: v["archiveSha256"] for v in self.receipt["packages"]}}
        for rows in ([], [current, deepcopy(current)],
                     [{**current, "comment": m.owner.approval_comment(stale)}],
                     [{**current, "user": {"id": 22}}], [{**current, "user": {"id": 51}}],
                     [{**current, "environments": [{"id": 41, "name": m.ENVIRONMENT_NAME}]}]):
            self.origins[self.history] = rows
            with self.subTest(rows=rows), self.assertRaises(ValueError):
                self.run_contract()

    def test_moving_current_main_after_approval_stops_the_handoff(self):
        def get(route):
            value = self.origins[route]
            if route == self.history:
                self.origins["/branches/main"] = {**self.origins["/branches/main"], "commit": {"sha": "c"*40}}
            return value
        with self.assertRaisesRegex(ValueError, "protected main no longer current"):
            self.run_contract(getter=get)


if __name__ == "__main__":
    unittest.main()
