# cspell:words importlib
"""Offline planner contracts. CI executes these; local execution is not required."""
import importlib.util
import json
from pathlib import Path
import unittest
from unittest import mock

spec = importlib.util.spec_from_file_location("impact", Path(__file__).with_name("ci-impact-plan.py"))
impact = importlib.util.module_from_spec(spec); spec.loader.exec_module(impact)
summary_spec = importlib.util.spec_from_file_location("impact_summary", Path(__file__).with_name("ci-impact-summary.py"))
summary = importlib.util.module_from_spec(summary_spec); summary_spec.loader.exec_module(summary)


def fixture():
    files = {"Cargo.toml": b'[workspace]\nmembers=["crates/*"]\n', "pnpm-workspace.yaml": b'packages:\n  - packages/*\n  - examples/*\n',
             "crates/base/Cargo.toml": b'[package]\nname="base"\nversion="1.0.0"\n',
             "crates/consumer/Cargo.toml": b'[package]\nname="consumer"\nversion="1.0.0"\n[dependencies]\nbase={path="../base"}\n',
             "crates/base/src/lib.rs": b'pub fn value() {}', "crates/base/tests/value.rs": b'#[test] fn example() {}'}
    for directory, name, deps in [("packages/a", "a", {}), ("packages/b", "b", {"a": "workspace:*"}), ("examples/demo", "demo", {"b": "workspace:*"})]:
        files[directory + "/package.json"] = json.dumps({"name": name, "dependencies": deps}).encode()
        files[directory + "/src/index.ts"] = b'export const value = 1;'
    return files


class PlannerContracts(unittest.TestCase):
    def test_shared_runtime_requests_all_examples_instead_of_a_partial_dependency_list(self):
        for directory in ["core", "cli", "runtime", "runtime-plugin-hmr", "runtime-plugin-import-meta"]:
            files = fixture()
            files["packages/" + directory + "/package.json"] = json.dumps({"name": "@farmfe/" + directory}).encode()
            changed = "packages/" + directory + "/src/index.ts"; files[changed] = b'export const shared = true'
            plan = impact.plan_changes(files, files, [{"status": "M", "paths": [changed]}])
            self.assertIn("examples", plan["proposedSuites"])
            self.assertEqual(plan["proposedExampleSelection"], {"mode": "all", "directories": [], "executionEnabled": False})
            self.assertEqual(plan["proposedExamples"], [])

    def test_real_example_name_collision_retains_verified_full_execution_identity(self):
        files = fixture()
        for directory in ["electron", "public-path", "react"]:
            files["examples/" + directory + "/package.json"] = b'{"name":"@farmfe-examples/react"}'
        with self.assertRaises(impact.WorkspaceIdentityError) as collision:
            impact.graph(files)
        environment = {"GITHUB_RUN_ID": "10", "GITHUB_RUN_ATTEMPT": "2", "GITHUB_REPOSITORY": "farm-fe/farm", "GITHUB_SHA": "c" * 40, "GITHUB_JOB": "cargo-test", "GITHUB_WORKFLOW_REF": "farm-fe/farm/.github/workflows/rust-test.yaml@refs/pull/2330/merge"}
        calls = []
        def read_git(*args):
            calls.append(args)
            if args == ("rev-parse", "HEAD"):
                return ("c" * 40).encode()
            if args in [("rev-parse", "HEAD^{tree}"), ("rev-parse", "c" * 40 + "^{tree}")]:
                return ("d" * 40).encode()
            if args == ("cat-file", "-t", "d" * 40):
                return b"tree\n"
            if args[0] == "rev-parse" and args[1].endswith("^{commit}"):
                value = args[1].removesuffix("^{commit}")
                if value in ["a" * 40, "b" * 40, "c" * 40]:
                    return value.encode()
            raise AssertionError("Unexpected fixture Git operation")
        with mock.patch.object(impact, "git", side_effect=AssertionError("Fixture must not access real Git")):
            plan = impact.conservative_plan("b" * 40, "a" * 40, "c" * 40, "unit", False, collision.exception, environment, read_git)
        self.assertIn(("cat-file", "-t", "d" * 40), calls)
        self.assertNotIn(("rev-parse", "d" * 40 + "^{commit}"), calls)
        self.assertTrue(plan["fullRequired"]); self.assertTrue(plan["planningFallback"])
        self.assertFalse(plan["sourceInputProofComplete"]); self.assertFalse(plan["skipAuthorized"]); self.assertFalse(plan["reuseAuthorized"])
        self.assertNotIn("fingerprints", plan)
        self.assertEqual(plan["reasons"][0]["errorClass"], "WorkspaceIdentityError")
        event = {"pull_request": {"head": {"sha": "a" * 40}, "base": {"sha": "b" * 40}}}
        rows = []
        for cell in impact.execution_cells("unit", False):
            observed = summary.receipt.make_observation(cell, "success", environment, event, "c" * 40, "d" * 40)
            rows.append(summary.receipt.bind_observation(plan, observed))
        self.assertTrue(summary.validate_cells(plan, rows, trusted_inventory=impact.execution_cells("unit", False), expected_identity=dict(plan)))
        with self.assertRaises(ValueError):
            impact.conservative_plan("b" * 40, "a" * 40, "c" * 40, "unit", False, collision.exception, environment, lambda *args: ("e" * 40).encode())
        with self.assertRaises(ValueError):
            impact.conservative_plan(None, "a" * 40, "c" * 40, "unit", False, collision.exception, environment, read_git)
        def wrong_commit_tree(*args):
            return ("e" * 40).encode() if args == ("rev-parse", "c" * 40 + "^{tree}") else read_git(*args)
        with self.assertRaisesRegex(ValueError, "tested commit tree"):
            impact.conservative_plan("b" * 40, "a" * 40, "c" * 40, "unit", False, collision.exception, environment, wrong_commit_tree)

    def test_commit_and_tree_types_are_distinct_and_use_the_injected_git(self):
        def read_git(*args):
            if args[0] == "cat-file":
                return b"tree" if args[-1] == "d" * 40 else b"commit"
            if args == ("rev-parse", "c" * 40 + "^{commit}"):
                return ("c" * 40).encode()
            raise ValueError("Fixture cannot peel a tree to commit")
        with mock.patch.object(impact, "git", side_effect=AssertionError("No real Git allowed")):
            self.assertEqual(impact.revision("c" * 40, read_git), "c" * 40)
            self.assertEqual(impact.tree_revision("d" * 40, read_git), "d" * 40)
            with self.assertRaises(ValueError):
                impact.revision("d" * 40, read_git)
            with self.assertRaises(ValueError):
                impact.tree_revision("c" * 40, read_git)
            with self.assertRaises(ValueError):
                impact.tree_revision("not-an-object", read_git)

    def test_dual_cargo_and_npm_directory_keeps_both_owners_and_native_impact(self):
        files = fixture(); files["pnpm-workspace.yaml"] += b'  - crates/*\n'
        files["crates/base/package.json"] = b'{"name":"dual-base","version":"1.0.0"}'
        self.assertEqual(impact.owners(impact.graph(files), "crates/base/src/lib.rs"), {"cargo:base", "npm:dual-base"})
        plan = impact.plan_changes(files, files, [{"status": "M", "paths": ["crates/base/src/lib.rs"]}])
        self.assertIn("cargo:base", plan["productNodes"]); self.assertIn("npm:dual-base", plan["productNodes"])
        self.assertIn("native", plan["proposedSuites"])

    def test_reverse_product_closure(self):
        files = fixture()
        plan = impact.plan_changes(files, files, [{"status": "M", "paths": ["crates/base/src/lib.rs"]}])
        self.assertEqual(plan["productNodes"], ["cargo:base", "cargo:consumer"])
        self.assertIn("native", plan["proposedSuites"])

    def test_test_named_files_are_product_inputs_until_proven_test_only(self):
        files = fixture(); nodes = impact.graph(files)
        hashes = {name: {"mode": "100644", "sha256": impact.digest(body.hex())} for name, body in files.items()}
        before = impact.fingerprints(nodes, hashes)
        hashes["crates/base/tests/value.rs"] = {"mode": "100644", "sha256": "changed"}
        after = impact.fingerprints(nodes, hashes)
        self.assertNotEqual(before["cargo:base"]["product"], after["cargo:base"]["product"])
        self.assertNotEqual(before["cargo:base"]["tests"], after["cargo:base"]["tests"])
        plan = impact.plan_changes(files, files, [{"status": "M", "paths": ["crates/base/tests/value.rs"]}])
        self.assertIn("native", plan["proposedSuites"])
        plan = impact.plan_changes(files, files, [{"status": "M", "paths": ["crates/base/tests/value.rs"]}], proven_test_inputs={"crates/base/tests/value.rs"})
        self.assertEqual(plan["productNodes"], [])
        self.assertEqual(plan["testNodes"], ["cargo:base", "cargo:consumer"])

    def test_actual_tested_merge_edges_and_inputs_are_not_lost(self):
        base = fixture(); tested = dict(base)
        tested["crates/merged/Cargo.toml"] = b'[package]\nname="merged"\nversion="1.0.0"\n[dependencies]\nbase={path="../base"}\n'
        plan = impact.plan_changes(base, base, [{"status": "M", "paths": ["crates/base/src/lib.rs"]}], extra_snapshots=[tested])
        self.assertIn("cargo:merged", plan["productNodes"])

    def test_generator_bridge_uses_the_actual_same_directory_cargo_manifest(self):
        files = fixture(); files["Cargo.toml"] = b'[workspace]\nmembers=["crates/*","packages/create-farm"]\n'
        files["packages/create-farm/Cargo.toml"] = b'[package]\nname="generator"\nversion="1.0.0"\n'
        files["packages/create-farm/package.json"] = b'{"name":"create-farm","version":"1.0.0"}'
        nodes = impact.graph(files)
        self.assertIn("cargo:generator", nodes["npm:create-farm"]["productDeps"])

    def test_acceptance_fingerprint_covers_runner_and_marks_dynamic_completeness_unproven(self):
        files = fixture(); nodes = impact.graph(files)
        hashes = {name: {"mode": "100644", "sha256": impact.digest(body.hex())} for name, body in files.items()}
        hashes["scripts/runner.mjs"] = {"mode": "100644", "sha256": "before"}
        before = impact.fingerprints(nodes, hashes)
        hashes["scripts/runner.mjs"]["sha256"] = "after"
        after = impact.fingerprints(nodes, hashes)
        self.assertNotEqual(before["cargo:base"]["tests"], after["cargo:base"]["tests"])
        self.assertFalse(after["cargo:base"]["acceptanceKeyComplete"])

    def test_removed_edge_survives_union(self):
        base = fixture(); head = dict(base)
        head["crates/consumer/Cargo.toml"] = b'[package]\nname="consumer"\nversion="1.0.0"\n'
        plan = impact.plan_changes(base, head, [{"status": "D", "paths": ["crates/base/src/lib.rs"]}])
        self.assertIn("cargo:consumer", plan["productNodes"])

    def test_rename_includes_old_and_new_owners_and_case(self):
        files = fixture()
        plan = impact.plan_changes(files, files, [{"status": "R100", "paths": ["packages/a/src/A.ts", "packages/b/src/a.ts"]}])
        self.assertEqual(plan["productNodes"], ["npm:a", "npm:b", "npm:demo"])
        self.assertIn("examples", plan["proposedSuites"])

    def test_unknown_and_policy_inputs_require_full(self):
        files = fixture()
        for name in ["unknown-file.txt", "Cargo.lock", ".github/workflows/ci.yaml", "scripts/ci-impact-plan.py", "patches/serde/src/lib.rs"]:
            with self.subTest(name=name):
                plan = impact.plan_changes(files, files, [{"status": "M", "paths": [name]}])
                self.assertTrue(plan["fullRequired"])
                self.assertEqual(plan["proposedSuites"], impact.ALL)

    def test_consumer_and_performance_drivers_select_their_acceptance(self):
        files = fixture()
        for name, suite in [("scripts/verify-v2-npm-consumer.mjs", "consumers"), ("scripts/performance-report.mjs", "performance")]:
            plan = impact.plan_changes(files, files, [{"status": "M", "paths": [name]}])
            self.assertFalse(plan["fullRequired"]); self.assertIn(suite, plan["proposedSuites"])
            self.assertNotIn("native", plan["proposedSuites"])

    def test_malformed_workspace_is_not_guessed(self):
        files = fixture(); files["pnpm-workspace.yaml"] += b'unknown: true\n'
        with self.assertRaises(ValueError):
            impact.graph(files)

    def test_stable_cell_gate_rejects_missing_skipped_stale_and_duplicate_results(self):
        plan = {"executionCells": ["native:ci:linux-x64-gnu", "consumers:linux-x64-gnu"], "headSHA": "a" * 40, "baseSHA": "b" * 40,
                "testedSHA": "c" * 40, "testedTree": "d" * 40, "runId": "10", "runAttempt": "2", "policy": impact.POLICY, "repository": "farm-fe/farm"}
        plan["planHash"] = impact.digest(plan)
        receipts = [{key: value for key, value in plan.items() if key != "executionCells"} | {"cell": cell, "status": "executed-success", "acceptanceInherited": False} for cell in plan["executionCells"]]
        trusted_inventory = list(plan["executionCells"]); expected_identity = dict(plan)
        self.assertTrue(summary.validate_cells(plan, receipts, trusted_inventory=trusted_inventory, expected_identity=expected_identity))
        for bad in [receipts[:1], receipts + [receipts[0]], [receipts[0], {**receipts[1], "status": "skipped"}], [receipts[0], {**receipts[1], "runAttempt": "1"}]]:
            with self.assertRaises(ValueError):
                summary.validate_cells(plan, bad, trusted_inventory=trusted_inventory, expected_identity=expected_identity)
        empty = {**plan, "executionCells": []}; empty["planHash"] = impact.digest({key: value for key, value in empty.items() if key != "planHash"})
        with self.assertRaises(ValueError):
            summary.validate_cells(empty, [], trusted_inventory=trusted_inventory, expected_identity=expected_identity)
        missing = {key: value for key, value in plan.items() if key != "headSHA"}; missing["planHash"] = impact.digest({key: value for key, value in missing.items() if key != "planHash"})
        with self.assertRaises(ValueError):
            summary.validate_cells(missing, receipts, trusted_inventory=trusted_inventory, expected_identity=expected_identity)
        changed = {**plan, "headSHA": "f" * 40}; changed["planHash"] = impact.digest({key: value for key, value in changed.items() if key != "planHash"})
        with self.assertRaises(ValueError):
            summary.validate_cells(changed, receipts, trusted_inventory=trusted_inventory, expected_identity=expected_identity)
        duplicate = {**plan, "executionCells": trusted_inventory + [trusted_inventory[0]]}; duplicate["planHash"] = impact.digest({key: value for key, value in duplicate.items() if key != "planHash"})
        with self.assertRaises(ValueError):
            summary.validate_cells(duplicate, receipts, trusted_inventory=trusted_inventory, expected_identity=expected_identity)
        with self.assertRaises(ValueError):
            summary.validate_cells(plan, receipts, trusted_inventory=[], expected_identity=expected_identity)

    def test_shadow_summary_does_not_treat_skips_as_success_or_publication(self):
        plan = {"mode": "shadow", "skipAuthorized": False, "reuseAuthorized": False, "planHash": "a" * 64}
        report = summary.summarize(plan, {"native": {"result": "skipped"}}, ["native", "consumer"])
        self.assertFalse(report["allExpectedJobsSuccessful"])
        self.assertFalse(report["publicationAuthorized"])
        self.assertFalse(report["dynamicMatrixVerified"])
        self.assertEqual([row["result"] for row in report["jobs"]], ["skipped", "missing"])

    def test_ci_scope_never_schedules_release_candidates(self):
        environment = {"GITHUB_REPOSITORY": "farm-fe/farm", "GITHUB_EVENT_NAME": "pull_request"}
        event = {"number": 2329, "pull_request": {"head": {"repo": {"full_name": "farm-fe/farm"}, "ref": "topic"}}}
        self.assertFalse(impact.release_candidate(event, environment))
        event["number"] = 3000
        self.assertFalse(impact.release_candidate(event, environment))
        event["pull_request"]["head"]["ref"] = "changeset-release/main"
        self.assertFalse(impact.release_candidate(event, environment))
        event["pull_request"]["head"]["repo"]["full_name"] = "fork/farm"
        for number in [2329, 3000]:
            event["number"] = number
            self.assertFalse(impact.release_candidate(event, environment))
        self.assertFalse(impact.release_candidate({}, {**environment, "GITHUB_EVENT_NAME": "workflow_dispatch"}))

    def test_failed_ci_native_and_suite_remain_required_cell_failures(self):
        rows = [{"cell": "native:ci:linux-x64-gnu", "status": "executed-failure"}, {"cell": "ts-tests:win32-x64-msvc", "status": "executed-cancelled"},
                {"cell": "performance:darwin-arm64", "status": "executed-failure"}, {"cell": "contracts", "status": "executed-success"}]
        outcomes, failures = summary.cell_outcomes(rows)
        self.assertEqual(len(outcomes), 4)
        self.assertEqual([row["cell"] for row in failures], ["native:ci:linux-x64-gnu", "ts-tests:win32-x64-msvc"])
        plan = {"mode": "shadow", "skipAuthorized": False, "reuseAuthorized": False}
        report = summary.summarize(plan, {"incremental-execution-gate": {"result": "failure"}}, ["incremental-execution-gate"])
        self.assertFalse(report["allExpectedJobsSuccessful"])
        self.assertFalse(report["publicationAuthorized"])

    def test_shadow_performance_remains_informational_without_relaxing_release_policy(self):
        plan = {"mode": "shadow", "skipAuthorized": False, "reuseAuthorized": False}
        report = summary.summarize(plan, {"native": {"result": "success"}, "performance-report": {"result": "failure"}}, ["native", "performance-report"])
        self.assertTrue(report["allExpectedJobsSuccessful"])
        self.assertEqual(report["requiredFailures"], [])
        self.assertEqual(report["informationalOutcomes"][0]["result"], "failure")

    def test_proposed_dag_is_closed_and_platform_downstream_is_independent(self):
        proposal = impact.proposed_dag(impact.ALL, ["examples/demo"])
        tasks = proposal["tasks"]
        self.assertFalse(proposal["executionEnabled"])
        for dependencies in tasks.values():
            self.assertTrue(set(dependencies).issubset(set(tasks)))
        self.assertEqual(tasks["examples:linux-x64-gnu"], ["native:ci:linux-x64-gnu"])
        self.assertNotIn("native-assembly", tasks["performance:linux-x64-gnu"])
        self.assertEqual(len(tasks["native-assembly"]), 9)


if __name__ == "__main__":
    unittest.main()
