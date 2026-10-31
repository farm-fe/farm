"""Read-only release entrypoint contracts with independent mock API facts."""
# cspell:words importlib
import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest import mock


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(file))
    value = importlib.util.module_from_spec(spec); spec.loader.exec_module(value)
    return value


entry = load("release_entry_contract", "incremental-release-entry.py")
fixtures = load("release_entry_fixture", "incremental-chain-plan.test.py")


class Contracts(unittest.TestCase):
    def setUp(self):
        files, family, source = fixtures.fixture()
        files['crates/core/src/lib.rs']+=b'\npub const VERSION: &str = "0.5.4";\n'
        files["docs/releases/v2-candidate.json"] = json.dumps(family).encode()
        self.source = {**source, "files": files, "modes": {name: "100644" for name in files}, "policyHash": "pinned-policy"}
        self.plan = entry.source_plan(self.source)
        self.root = Path(tempfile.mkdtemp(prefix="farm-release-entry-contract-"))
        nodes, issues = entry.chain.repository_graph(files)
        family = entry.chain.independent_family(files, nodes)
        rows = []
        for index, (identity, row) in enumerate(sorted(family.items())):
            body = identity.encode(); file = str(index) + ".archive"; (self.root / file).write_bytes(body)
            rows.append({"kind": row["kind"], "name": row["name"], "version": row["version"], "archive": file,
                         "archiveBytes": len(body), "archiveSha256": hashlib.sha256(body).hexdigest()})
        self.manifest = {"chain": "release", "profile": "release", "namespace": "farm-release-v1", "sourceSHA": source["sha"],
                         "sourceTree": source["tree"], "producerRunId": 10, "producerAttempt": 2, "baseline": None,
                         "planHash": self.plan["planHash"], "packages": rows}
        self.manifest.update({'family':self.plan['fullFamily'],'rebuild':self.plan['build'],'versionRequired':self.plan['versionRequired'],'publish':self.plan['publish']})
        self.run = {"id": 10, "run_attempt": 2, "repository": {"id": 5}, "head_repository": {"id": 5}, "path": entry.WORKFLOW,
                    "head_sha": source["sha"], "event": "workflow_dispatch", "head_branch": "main", "status": "completed", "conclusion": "success"}
        self.jobs = [{"id": index, "name": name, "status": "completed", "conclusion": "success", "steps": [{"name": step,
                     "status": "completed", "conclusion": "success", "started_at": "2026-01-01T00:00:00Z", "completed_at": "2026-01-01T00:01:00Z"}]} for index, name, step in [
                     (11, entry.PRODUCER, "Upload independent release seal"), (12, entry.CONSUMER, "Upload independent release public acceptance")]]
        self.repository = {"id": 5, "default_branch": "main"}
        self.jobs += [{"id": index, "name": "Independent release candidate consumer (" + platform + ")", "status": "completed", "conclusion": "success"}
                      for index, platform in enumerate(["ubuntu-24.04", "macos-latest", "windows-latest"], 20)]
        self.transport = mock.Mock()
        def get(route):
            if route == "": return self.repository
            if route == "/branches/main": return {"commit": {"sha": source["sha"]}}
            if route == "/actions/runs/10/attempts/2": return self.run
            raise AssertionError("Unexpected API route")
        self.transport.get.side_effect = get
        manifest_hash = entry.chain.digest(self.manifest)
        self.seal = {"kind": "release-seal", "profile": "release", "namespace": "farm-release-v1", "manifestHash": manifest_hash,
                     "producerJobId": 11, "sourceTree": source["tree"], "packageInputs": entry.evidence.package_input_hashes(self.source)}
        self.public = {"manifestHash": manifest_hash, "producerJobId": 12, "sourceTree": source["tree"], "planHash": self.plan["planHash"],
                       "packages": {row["kind"] + ":" + row["name"]: row["archiveSha256"] for row in rows},
                       "registryInstall": True, "nativeABI": True, "latestTargets": True}
        self.artifact = {"created_at": "2026-01-01T00:00:30Z"}
        self.verifier = entry.ReleaseVerifier(self.transport, lambda sha: copy.deepcopy(self.source))

    def verify(self):
        with mock.patch.object(entry.execution, "jobs_for", return_value=self.jobs), mock.patch.object(entry.execution, "fetch_json_artifact",
                side_effect=[(self.seal, self.artifact), (self.public, self.artifact)]):
            return self.verifier.verify(self.manifest, self.root)

    def test_complete_read_only_proof_checks_actual_archive_bytes(self):
        result = self.verify()
        self.assertTrue(result["verified"]); self.assertFalse(result["publicationExecuted"])
        self.assertEqual(result["producerRunId"], 10)

    def test_ci_profile_and_namespace_are_rejected(self):
        for key, value in [("profile", "ci"), ("namespace", "farm-ci-v1"), ("chain", "ci")]:
            original = self.manifest[key]; self.manifest[key] = value
            with self.assertRaisesRegex(ValueError, "CI bytes"):
                self.verify()
            self.manifest[key] = original

    def test_fork_pr_wrong_attempt_wrong_workflow_failed_run_are_rejected(self):
        for key, value in [("event", "pull_request"), ("run_attempt", 1), ("path", ".github/workflows/ci.yaml"),
                           ("conclusion", "failure"), ("head_repository", {"id": 99})]:
            original = self.run[key]; self.run[key] = value
            with self.assertRaises(ValueError): self.verify()
            self.run[key] = original

    def test_omitted_extra_or_reversioned_package_is_rejected(self):
        original = copy.deepcopy(self.manifest["packages"])
        for rows in [original[:-1], original + [original[0]], [{**original[0], "version": "99.0.0"}, *original[1:]]]:
            self.manifest["packages"] = rows
            with self.assertRaises(ValueError): self.verify()
        self.manifest["packages"] = original

    def test_plan_tree_and_producer_job_binding_cannot_be_self_asserted(self):
        self.manifest["planHash"] = "f" * 64
        with self.assertRaisesRegex(ValueError, "plan differs"): self.verify()
        self.manifest["planHash"] = self.plan["planHash"]
        self.seal["producerJobId"] = 99
        with self.assertRaisesRegex(ValueError, "seal binding"): self.verify()

    def test_missing_public_package_or_required_acceptance_is_rejected(self):
        self.public["latestTargets"] = False
        with self.assertRaisesRegex(ValueError, "acceptance missing"): self.verify()
        self.public["latestTargets"] = True
        self.public["packages"].pop(next(iter(self.public["packages"])))
        with self.assertRaisesRegex(ValueError, "coverage differs"): self.verify()

    def test_changed_archive_and_parent_escape_are_rejected(self):
        row = self.manifest["packages"][0]; (self.root / row["archive"]).write_bytes(b"different archive")
        with self.assertRaises(ValueError): self.verify()
        row["archive"] = "../outside.archive"
        with self.assertRaisesRegex(ValueError, "canonical basename"): self.verify()

    def test_live_transport_refuses_pr_local_and_fake_workflow_roles(self):
        values = {"GITHUB_ACTIONS": "true", "GITHUB_REF": "refs/heads/main", "GITHUB_EVENT_NAME": "workflow_dispatch",
                  "GITHUB_REPOSITORY": "farm-fe/farm", "GITHUB_WORKFLOW_REF": "farm-fe/farm/" + entry.WORKFLOW + "@refs/heads/main", "GH_TOKEN": "mock-not-used"}
        for key, value in [("GITHUB_ACTIONS", "false"), ("GITHUB_EVENT_NAME", "pull_request"), ("GITHUB_REF", "refs/pull/2330/merge"),
                           ("GITHUB_WORKFLOW_REF", "farm-fe/farm/" + entry.WORKFLOW + "@refs/heads/main-extra")]:
            changed = {**values, key: value}
            with self.assertRaises(ValueError): entry.actions_transport(changed)

    def test_uploaded_later_baseline_is_not_trusted_as_success(self):
        self.manifest["baseline"] = "f" * 64
        with self.assertRaisesRegex(PermissionError, "only in Actions"): self.verify()

    def test_frozen_main_policy_survives_descendant_policy_change_but_not_unrelated_history(self):
        original=self.transport.get.side_effect;later={**self.source,'sha':'c'*40,'policyHash':'later-approved-policy'}
        self.verifier.read_source=lambda sha:later if sha==later['sha'] else self.source
        comparison={'status':'ahead','base_commit':{'sha':self.source['sha']},'merge_base_commit':{'sha':self.source['sha']}}
        def get(route):
            if route=='/branches/main':return {'commit':{'sha':later['sha']}}
            if route.startswith('/compare/'):return comparison
            return original(route)
        self.transport.get.side_effect=get
        self.assertTrue(self.verify()['verified'])
        comparison['merge_base_commit']['sha']='f'*40
        with self.assertRaisesRegex(ValueError,'trusted main history'):self.verify()

    def test_prepublish_uses_candidate_acceptance_without_public_registry_cycle(self):
        self.run.update({"status": "in_progress", "conclusion": None})
        self.jobs = [job for job in self.jobs if job["name"] != entry.CONSUMER]
        with mock.patch.object(entry.execution, "jobs_for", return_value=self.jobs), mock.patch.object(entry.execution, "fetch_json_artifact",
                return_value=(self.seal, self.artifact)) as fetch:
            result = self.verifier.verify(self.manifest, self.root, phase="prepublish", current_run={"runId": 10, "attempt": 2})
        self.assertTrue(result["prepublicationVerified"]); self.assertFalse(result["verified"])
        self.assertEqual(fetch.call_count, 1)

    def test_running_attempt_cannot_borrow_another_current_run_identity(self):
        self.run.update({"status": "in_progress", "conclusion": None})
        with self.assertRaisesRegex(ValueError, "has not succeeded"):
            self.verifier.verify(self.manifest, self.root, phase="prepublish", current_run={"runId": 99, "attempt": 2})

    def test_missing_candidate_platform_blocks_before_first_registry_write(self):
        self.jobs = [job for job in self.jobs if job["name"] != "Independent release candidate consumer (windows-latest)"]
        with self.assertRaisesRegex(ValueError, "Missing or ambiguous baseline job"):
            self.verify()


if __name__ == "__main__":
    unittest.main(verbosity=2)
