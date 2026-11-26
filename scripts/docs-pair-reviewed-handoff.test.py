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
        self.actor = 22
        repo = {"id": m.REPOSITORY_ID, "full_name": "farm-fe/farm"}
        base = {"head_sha": sha, "head_branch": "main", "run_attempt": 1, "event": "workflow_dispatch",
                "actor": {"id": 22, "type": "User"}, "repository": repo, "head_repository": deepcopy(repo)}
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
        self.history = "/actions/workflows/21/runs?branch=main&event=workflow_dispatch&per_page=2"
        self.origins[self.history] = {"total_count": 1, "workflow_runs": [{"id": 20, "run_attempt": 1}]}
        self.origins["/actions/workflows/docs-pair-publish.yaml"] = {"id": 21, "path": m.WRITER, "state": "active"}
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
    def run_contract(self, getter=None):
        def get(route):
            self.calls.append(route)
            self.assertIn(route, self.origins)
            return self.origins[route]
        def download(artifact, limit):
            self.assertEqual((artifact, limit), (30, m.MAX_ZIP))
            self.calls.append("download-frozen-30")
            return self.zip
        return m.read_reviewed_handoff(self.selection, self.actor, get=get if getter is None else getter, download=download)

    def test_valid_split_role_order_and_exact_result(self):
        result = self.run_contract()
        self.assertEqual(result, {"producerJobId": 19, "writerWorkflowId": 21, "artifactId": 30, "receipt": self.receipt, "archives": self.bodies})
        self.assertEqual(self.calls[-4:], ["download-frozen-30", "/branches/main", f"/git/commits/{self.selection['sourceSHA']}", self.history])

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
            ("/actions/runs/20/attempts/1", "actor", {"id": 20, "type": "User"}),
            ("/actions/runs/20", "actor", {"id": 22, "type": "Bot"})]:
            old = self.origins[route][key]
            self.origins[route][key] = value
            with self.subTest(route=route, key=key), self.assertRaises(ValueError):
                self.run_contract()
            self.origins[route][key] = old
        self.assertNotIn("/actions/artifacts/30", self.calls)
        self.assertNotIn(self.history, self.calls)

    def test_dispatchers_cannot_alias_canonical_roles(self):
        for obj, key, value in [
            (self.selection, "writerRun", 10), (self.selection, "producerRun", True), (self.selection, "tag", "latest")]:
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
        self.assertNotIn("download-frozen-30", self.calls)

    def test_mutated_zip_resealed_repack_and_extra_entry_never_reach_approval(self):
        original = self.zip
        self.zip = original + b"non-original"
        with self.assertRaisesRegex(ValueError, "ZIP bytes differ"):
            self.run_contract()
        self.freeze(archive_changes={m.PACKAGE_ORDER[0]: b"bad"})
        with self.assertRaisesRegex(ValueError, "original package bytes"):
            self.run_contract()
        self.freeze(extra=True)
        with self.assertRaisesRegex(ValueError, "exact original files"):
            self.run_contract()
        self.assertEqual(self.calls.count(self.history), 3)

    def test_missing_duplicate_or_prior_history_never_allows_new_upload(self):
        real = self.origins[self.history]
        for rows in ({"total_count": 0, "workflow_runs": []},
                     {**real, "total_count": 2},
                     {"total_count": 1, "workflow_runs": [{"id": 19, "run_attempt": 1}]},
                     {"total_count": 1, "workflow_runs": [{"id": 20, "run_attempt": 2}]}):
            self.origins[self.history] = rows
            with self.subTest(rows=rows), self.assertRaisesRegex(ValueError, "publisher history"):
                self.run_contract()
        self.assertNotIn("download-frozen-30", self.calls)

    def test_wrong_runtime_actor_or_registered_workflow_never_reaches_archive(self):
        self.actor = 23
        with self.assertRaisesRegex(ValueError, "initiating actor"):
            self.run_contract()
        self.actor = 22
        self.origins["/actions/workflows/docs-pair-publish.yaml"]["id"] = m.PRODUCER_ID
        with self.assertRaisesRegex(ValueError, "registered writer"):
            self.run_contract()
        self.assertNotIn("download-frozen-30", self.calls)

    def test_moving_current_main_after_archive_stops_the_handoff(self):
        def get(route):
            value = self.origins[route]
            if route == "/actions/artifacts/30":
                self.origins["/branches/main"] = {**self.origins["/branches/main"], "commit": {"sha": "c"*40}}
            return value
        with self.assertRaisesRegex(ValueError, "protected main no longer current"):
            self.run_contract(getter=get)


if __name__ == "__main__":
    unittest.main()
