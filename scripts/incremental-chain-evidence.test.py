"""Offline evidence contracts. Only remote CI executes these tests."""
# cspell:words importlib
import copy
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import unittest
import zipfile


def module(name, file):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(file))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


evidence = module("contract_chain_evidence", "incremental-chain-evidence.py")
fixtures = module("contract_chain_fixtures", "incremental-chain-plan.test.py")
runtime = module("contract_chain_runtime", "incremental-chain-runtime.py")


def fixture(mode="ci"):
    files, family, _ = fixtures.fixture()
    files["docs/releases/v2-candidate.json"] = json.dumps(family).encode()
    ledger, _ = fixtures.baseline(files, family, mode)
    source = {"sha": ledger["sourceSHA"], "tree": ledger["sourceTree"], "files": files, "modes": fixtures.modes(files)}
    source["policyHash"] = evidence.policy_hash(files, source["modes"])
    producer_name, upload_name = evidence.PRODUCERS[mode]
    ledger.update({"policyHash": source["policyHash"], "requiredJobs": [producer_name, "Public registry consumers"],
                   "producerJob": producer_name, "registryChecksums": {key: row["archiveSha256"] for key, row in ledger["packages"].items()}})
    inputs = evidence.package_input_hashes(source)
    for key, row in ledger["packages"].items():
        row["buildContext"] = {"chain": mode, "profile": mode, "sourceInputHash": inputs[key], "recipeHash": source["policyHash"],
                               "abi": family["abi"], "completeMeasuredContext": True, "toolchainHash": "f" * 64,
                               "dependencyHash": evidence.chain_plan.digest({}), "flagsHash": "a" * 64}
    repository = {"id": 1, "default_branch": "main"}
    run = {"id": 2, "run_attempt": 1, "repository": {"id": 1}, "head_repository": {"id": 1},
           "path": evidence.WORKFLOWS[mode], "event": "push", "head_branch": "main", "status": "completed", "conclusion": "success",
           "head_sha": source["sha"]}
    artifact = {"id": 3, "name": "farm-" + mode + "-baseline-" + source["sha"] + "-1", "workflow_run": {"id": 2},
                "expired": False, "created_at": "2026-10-31T01:02:00Z", "digest": "sha256:" + "a" * 64}
    jobs = [{"id": index + 10, "run_id": 2, "name": name, "status": "completed", "conclusion": "success",
             "started_at": "2026-10-31T01:00:00Z", "completed_at": "2026-10-31T01:03:00Z",
             "steps": [{"name": upload_name, "status": "completed", "conclusion": "success",
                        "started_at": "2026-10-31T01:01:00Z", "completed_at": "2026-10-31T01:03:00Z"}]}
            for index, name in enumerate(ledger["requiredJobs"])]
    checksums = copy.deepcopy(ledger["registryChecksums"])
    measured = {key: copy.deepcopy(row["buildContext"]) for key, row in ledger["packages"].items()}
    return ledger, mode, repository, run, artifact, jobs, source, source, ledger["requiredJobs"], checksums, measured


class EvidenceContracts(unittest.TestCase):
    def test_lookup_fetches_attempt_job_artifact_bytes_and_rechecks_main(self):
        values = fixture()
        ledger, mode, repository, run, artifact, jobs, source, _, required, _, measured = values
        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, "w") as archive:
            archive.writestr("ledger.json", json.dumps(ledger))
        body = buffer.getvalue()
        artifact.update({"size_in_bytes": len(body), "digest": "sha256:" + hashlib.sha256(body).hexdigest()})
        policy_sha = "f" * 40
        class Transport:
            branch_calls = 0
            move_main = False
            def get(self, route):
                if route == "":
                    return repository
                if route.startswith("/branches/"):
                    self.branch_calls += 1
                    return {"commit": {"sha": "e" * 40 if self.move_main and self.branch_calls > 1 else policy_sha}}
                if route.startswith("/actions/workflows/"):
                    return {"workflow_runs": [run]}
                if "/attempts/1/jobs?" in route:
                    return {"jobs": jobs}
                if route.endswith("/attempts/1"):
                    return run
                if "/artifacts?" in route:
                    self.filtered = "&name=farm-ci-baseline-" in route
                    return {"artifacts": [artifact], "total_count": 1}
                if route == "/actions/artifacts/3":
                    return artifact
                raise AssertionError("Unexpected evidence route")
            def archive(self, identity):
                if identity != 3:
                    raise AssertionError("Wrong artifact ID")
                return body
        def read_source(commit):
            result = copy.deepcopy(source); result["sha"] = commit
            return result
        transport = Transport()
        adapter = evidence.BaselineAdapter(transport, 1, policy_sha, mode, required, read_source,
                                          measurement_reader=lambda *_: measured)
        result = adapter.lookup()
        self.assertEqual(result["anchor"]["artifactId"], 3)
        self.assertTrue(transport.filtered)
        self.assertEqual(transport.branch_calls, 2)
        transport = Transport(); transport.move_main = True
        adapter = evidence.BaselineAdapter(transport, 1, policy_sha, mode, required, read_source,
                                          measurement_reader=lambda *_: measured)
        with self.assertRaisesRegex(ValueError, "moved during"):
            adapter.lookup()

    def test_valid_baseline_anchor_is_derived_from_independent_inputs(self):
        values = fixture()
        anchor = evidence.verify_ledger(*values)
        self.assertTrue(anchor["verified"])
        self.assertEqual(anchor["artifactId"], 3)

    def test_self_declared_verified_cannot_override_github(self):
        values = list(fixture()); values[0]["verified"] = True
        values[3]["head_repository"]["id"] = 999
        with self.assertRaisesRegex(ValueError, "repository"):
            evidence.verify_ledger(*values)

    def test_wrong_event_profile_workflow_tree_attempt_and_policy_are_rejected(self):
        mutations = [(3, "event", "pull_request"), (0, "profile", "release"),
                     (3, "path", evidence.WORKFLOWS["release"]), (0, "sourceTree", "e" * 40),
                     (0, "runAttempt", 2), (0, "policyHash", "b" * 64)]
        for index, key, value in mutations:
            values = list(fixture()); values[index][key] = value
            with self.assertRaises(ValueError):
                evidence.verify_ledger(*values)

    def test_bad_acceptance_or_expired_artifact_cannot_become_a_baseline(self):
        for state in ["failure", "cancelled", "skipped"]:
            values = list(fixture()); values[5][1]["conclusion"] = state
            with self.assertRaisesRegex(ValueError, "succeed"):
                evidence.verify_ledger(*values)
        values = list(fixture()); values[4]["expired"] = True
        with self.assertRaisesRegex(ValueError, "expired"):
            evidence.verify_ledger(*values)

    def test_release_requires_public_consumers_and_registry_records(self):
        values = list(fixture("release")); values[0]["registryVerified"] = False
        with self.assertRaisesRegex(ValueError, "public consumer"):
            evidence.verify_ledger(*values)

    def test_forged_both_registry_fields_do_not_replace_live_checksums(self):
        values = list(fixture("release")); key = next(iter(values[0]["packages"]))
        values[0]["packages"][key]["archiveSha256"] = "b" * 64
        values[0]["registryChecksums"][key] = "b" * 64
        with self.assertRaisesRegex(ValueError, "registry checksum"):
            evidence.verify_ledger(*values)

    def test_artifact_cannot_choose_a_different_successful_producer(self):
        values = list(fixture()); values[0]["producerJob"] = "Public registry consumers"
        with self.assertRaisesRegex(ValueError, "own producer"):
            evidence.verify_ledger(*values)
        values = list(fixture()); values[5][0]["steps"] = []
        with self.assertRaisesRegex(ValueError, "upload step"):
            evidence.verify_ledger(*values)

    def test_build_input_and_incomplete_runtime_proof_are_rejected(self):
        for key, value in [("sourceInputHash", "a" * 64), ("completeMeasuredContext", False), ("chain", "release")]:
            values = list(fixture()); next(iter(values[0]["packages"].values()))["buildContext"][key] = value
            with self.assertRaises(ValueError):
                evidence.verify_ledger(*values)
        values = list(fixture()); values[10] = None
        with self.assertRaisesRegex(ValueError, "producer measurement"):
            evidence.verify_ledger(*values)

    def test_artifact_has_one_bounded_regular_json_member_and_verified_digest(self):
        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, "w") as archive:
            archive.writestr("ledger.json", '{"schemaVersion":1}')
        body = buffer.getvalue(); meta = {"size_in_bytes": len(body), "digest": "sha256:" + hashlib.sha256(body).hexdigest()}
        self.assertEqual(evidence.ledger_member(body, meta), {"schemaVersion": 1})
        meta["digest"] = "sha256:" + "0" * 64
        with self.assertRaisesRegex(ValueError, "digest"):
            evidence.ledger_member(body, meta)
        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, "w") as archive:
            archive.writestr("../ledger.json", '{}')
        body = buffer.getvalue(); meta = {"size_in_bytes": len(body), "digest": "sha256:" + hashlib.sha256(body).hexdigest()}
        with self.assertRaisesRegex(ValueError, "members"):
            evidence.ledger_member(body, meta)

    def test_a_changed_baseline_binding_is_rejected_inside_publication_lock(self):
        with self.assertRaisesRegex(ValueError, "changed"):
            evidence.recheck_binding({"bindingHash": "a"}, {"bindingHash": "b"})
        with self.assertRaisesRegex(ValueError, "changed"):
            evidence.recheck_binding(None, {"bindingHash": "a"})

    def test_access_denied_selects_full_current_chain_without_legacy_reuse(self):
        class Denied:
            def lookup(self):
                raise PermissionError("denied")
        files, family, source = fixtures.fixture()
        plan, found = evidence.choose_plan(Denied(), "ci", source, files, fixtures.modes(files), family, [])
        self.assertIsNone(found)
        self.assertTrue(plan["fullBuild"])
        self.assertIsNone(plan["baselineBinding"])
        self.assertFalse(plan["crossChainReuseAllowed"])

    def test_execution_gate_rejects_skip_empty_count_and_foreign_plan(self):
        run = {"runId": 1, "runAttempt": 1}
        plan = {"currentRun": run, "sourceSHA": "a" * 40, "sourceTree": "b" * 40}
        plan["planHash"] = evidence.chain_plan.digest(plan)
        row = {"cell": "examples:linux", "planHash": plan["planHash"], "sourceSHA": plan["sourceSHA"], "sourceTree": plan["sourceTree"],
               "run": run, "status": "executed-success", "executedCount": 1}
        jobs = {row["cell"]: {"run": run, "status": "completed", "conclusion": "success", "jobId": 3}}
        inventory_hash = "c" * 64
        selection = {"cell": row["cell"], "inventoryHash": inventory_hash, "ids": ["example"],
                     "sourceSHA": plan["sourceSHA"], "sourceTree": plan["sourceTree"], "run": run, "executionPlanHash": plan["planHash"]}
        counts = {row["cell"]: {"producerJobId": 3, "run": run, "sourceSHA": plan["sourceSHA"], "sourceTree": plan["sourceTree"],
                               "artifactDigestVerified": True, "selectedIds": ["example"], "executedIds": ["example"], "failed": 0, "skipped": 0,
                               "executionPlanHash": plan["planHash"], "suiteInventoryHash": inventory_hash, "selectionHash": evidence.chain_plan.digest(selection)}}
        # The injected discovery boundary never reads the receipt or count fields.
        discovery = lambda *_: {row["cell"]: {"ids": ["example"], "inventoryHash": inventory_hash}}
        proof = {"verified_jobs": jobs, "semantic_counts": counts, "suite_discovery": discovery}
        self.assertEqual(runtime.validate_execution_results(plan, [row], run, [row["cell"]], **proof)["executedCells"], 1)
        for key, value in [("status", "skipped"), ("executedCount", 0), ("planHash", "0" * 64)]:
            bad = copy.deepcopy(row); bad[key] = value
            with self.assertRaises(ValueError):
                runtime.validate_execution_results(plan, [bad], run, [row["cell"]], **proof)
        with self.assertRaisesRegex(ValueError, "inventory"):
            runtime.validate_execution_results(plan, [], run, [], **proof)
        with self.assertRaisesRegex(ValueError, "Independent successful job"):
            runtime.validate_execution_results(plan, [row], run, [row["cell"]], verified_jobs={}, semantic_counts={}, suite_discovery=discovery)
        bad_counts = copy.deepcopy(counts); bad_counts[row["cell"]]["executedIds"] = []
        with self.assertRaisesRegex(ValueError, "semantic suite inventory"):
            runtime.validate_execution_results(plan, [row], run, [row["cell"]], verified_jobs=jobs, semantic_counts=bad_counts, suite_discovery=discovery)
        another = copy.deepcopy(plan); another.pop("planHash"); another["expandedPlan"] = True
        another["planHash"] = evidence.chain_plan.digest(another)
        rebound = copy.deepcopy(row); rebound["planHash"] = another["planHash"]
        with self.assertRaisesRegex(ValueError, "another execution plan"):
            runtime.validate_execution_results(another, [rebound], run, [row["cell"]], **proof)
        expanded = lambda *_: {row["cell"]: {"ids": ["example", "other"], "inventoryHash": inventory_hash}}
        short = copy.deepcopy(counts)
        short[row["cell"]]["selectionHash"] = evidence.chain_plan.digest({**selection, "ids": ["example", "other"]})
        with self.assertRaisesRegex(ValueError, "semantic suite inventory"):
            runtime.validate_execution_results(plan, [row], run, [row["cell"]], verified_jobs=jobs, semantic_counts=short, suite_discovery=expanded)
        old_attempt = copy.deepcopy(counts); old_attempt[row["cell"]]["run"] = {**run, "runAttempt": 2}
        with self.assertRaisesRegex(ValueError, "artifact provenance"):
            runtime.validate_execution_results(plan, [row], run, [row["cell"]], verified_jobs=jobs, semantic_counts=old_attempt, suite_discovery=discovery)


if __name__ == "__main__":
    unittest.main()
