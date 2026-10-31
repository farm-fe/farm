"""Read-only real-repository oracle with explicitly simulated baseline evidence."""
# cspell:words importlib
import copy
import importlib.util
import json
import os
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("real_repo_chain", Path(__file__).with_name("incremental-chain-plan.py"))
chain = importlib.util.module_from_spec(spec); spec.loader.exec_module(chain)


def reverse_oracle(nodes, roots, edge):
    reverse = {key: set() for key in nodes}
    for consumer, node in nodes.items():
        for dependency in node[edge]:
            reverse[dependency].add(consumer)
    pending, result = list(roots), set()
    while pending:
        node = pending.pop(0)
        if node not in result:
            result.add(node); pending += sorted(reverse[node] - result)
    return result


class RealRepositoryOracle(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        sha = chain.impact.git("rev-parse", "HEAD").decode().strip()
        cls.files, hashes = chain.impact.snapshot(sha)
        cls.modes = {file: row["mode"] for file, row in hashes.items()}
        cls.source = {"sha": sha, "tree": chain.impact.git("rev-parse", "HEAD^{tree}").decode().strip()}
        cls.family = json.loads(cls.files["docs/releases/v2-candidate.json"])
        cls.graph, issues = chain.repository_graph(cls.files)
        if issues:
            raise AssertionError("Real repository contains unresolved graph edges: " + repr(issues))
        cls.records = chain.family_records(cls.family)
        cls.examples = {key for key, node in cls.graph.items() if node["kind"] == "npm" and node["directory"].startswith("examples/")}
        cls.rows = []

    @classmethod
    def tearDownClass(cls):
        output = os.environ.get("FARM_INCREMENTAL_REPORT")
        if output:
            Path(output).write_text(json.dumps({"simulation": True, "liveBaselineAuthorized": False, "source": cls.source,
                                               "familyRecords": len(cls.records), "examples": len(cls.examples), "cases": cls.rows}, indent=2) + "\n", encoding="utf-8")

    def baseline(self, mode):
        ledger = {"schemaVersion": 1, "chain": mode, "profile": mode, "policy": chain.POLICY, "status": "success", "abi": self.family["abi"],
                  "repositoryId": 1, "runId": 2, "runAttempt": 1, "sourceSHA": self.source["sha"], "sourceTree": self.source["tree"],
                  "sourceSnapshotHash": chain.digest(chain.snapshot_records(self.files, self.modes)), "registryVerified": True,
                  "packages": {key: {"identity": key, "version": row["version"], "archiveSha256": "b" * 64} for key, row in self.records.items()}}
        anchor = {key: ledger[key] for key in ["chain", "repositoryId", "runId", "runAttempt", "sourceSHA", "sourceTree"]}
        anchor.update({"verified": True, "ledgerSha256": chain.digest(ledger), "registryChecksums": {key: row["archiveSha256"] for key, row in ledger["packages"].items()}})
        return ledger, anchor

    def select(self, files, family=None, mode="ci", modes=None):
        ledger, anchor = self.baseline(mode)
        return chain.plan_chain(mode, self.source, files, family or self.family, [], ledger, self.files, anchor,
                                source_modes=modes or self.modes, baseline_modes=self.modes)

    def projection(self, affected):
        directories = {self.graph[key]["directory"] for key in affected if self.graph[key]["kind"] == "npm"}
        return {key for key, row in self.records.items() if row["kind"] + ":" + str(Path(row["manifestPath"]).parent).replace("\\", "/") in affected
                or row.get("owner") and str(Path(self.records["npm:" + row["owner"]]["manifestPath"]).parent).replace("\\", "/") in directories}

    def check_case(self, name, plan, build, tests):
        self.rows.append({"name": name, "fullBuild": plan["fullBuild"], "actualBuild": plan["build"], "expectedBuild": sorted(build),
                          "actualTests": plan["testNodes"], "expectedTests": sorted(tests), "publish": plan["publish"], "versionRequired": plan["versionRequired"]})
        self.assertEqual(set(plan["build"]), set(build))
        self.assertEqual(set(plan["testNodes"]), set(tests))
        self.assertFalse(plan["executionAuthorized"]); self.assertFalse(plan["publicationAuthorized"])

    def test_actual_repository_no_change_is_not_full(self):
        plan = self.select(self.files)
        self.check_case("no-change", plan, set(), set())
        self.assertFalse(plan["fullBuild"]); self.assertEqual(plan["publish"], [])

    def test_actual_shared_core_cli_and_runtime_cover_all_examples(self):
        for directory in ["packages/core", "packages/cli", "packages/runtime", "packages/runtime-plugin-hmr", "packages/runtime-plugin-import-meta"]:
            candidates = sorted(file for file in self.files if file.startswith(directory + "/src/") and file.endswith((".ts", ".js")))
            self.assertTrue(candidates, directory)
            files = dict(self.files); files[candidates[0]] += b'\n// simulated source mutation\n'
            root = {"npm:" + directory}
            products = reverse_oracle(self.graph, root, "productDeps")
            expected_tests = reverse_oracle(self.graph, root, "testDeps") | self.examples
            if "npm:website" in self.graph:
                expected_tests.add("npm:website")
            plan = self.select(files)
            self.check_case(directory, plan, self.projection(products), expected_tests)
            self.assertFalse(plan["fullBuild"])
            self.assertLess(len(plan["build"]), len(self.records))
            self.assertEqual(self.examples & set(plan["testNodes"]), self.examples)

    def test_actual_leaf_example_selects_only_its_reverse_dependents(self):
        path = sorted(file for file in self.files if file.startswith("examples/") and file.endswith("/e2e.spec.mjs"))[0]
        key = "npm:" + "/".join(path.split("/")[:2])
        files = dict(self.files); files[path] += b'\n// simulated browser contract change\n'
        plan = self.select(files)
        products = reverse_oracle(self.graph, {key}, "productDeps")
        self.check_case("leaf-e2e:" + path, plan, self.projection(products), reverse_oracle(self.graph, {key}, "testDeps"))
        self.assertEqual(plan["build"], []); self.assertFalse(plan["fullBuild"])

    def test_actual_js_dts_source_versions_and_inventory_keep_exact_publish_delta(self):
        directory = "js-plugins/dts"; key = "npm:" + directory
        path = sorted(file for file in self.files if file.startswith(directory + "/src/") and file.endswith(".ts"))[0]
        files = dict(self.files); files[path] += b'\n// simulated JS plugin mutation\n'
        affected = reverse_oracle(self.graph, {key}, "productDeps"); expected = self.projection(affected)
        self.assertTrue(expected); self.assertFalse(any(key.startswith("cargo:") for key in expected))
        family = copy.deepcopy(self.family)
        versions = {}
        for row in family["npm"]["primary"]:
            if "npm:" + row["name"] in expected:
                major, minor, patch = row["version"].split("."); row["version"] = major + "." + minor + "." + str(int(patch) + 1)
                versions[row["name"]] = row["version"]
                package = json.loads(files[row["manifestPath"]]); package["version"] = row["version"]
                files[row["manifestPath"]] = json.dumps(package).encode()
        for row in family["npm"]["payloads"]:
            if row["owner"] in versions:
                row["version"] = versions[row["owner"]]
                package = json.loads(files[row["manifestPath"]]); package["version"] = row["version"]
                files[row["manifestPath"]] = json.dumps(package).encode()
                owner = next(item for item in family["npm"]["primary"] if item["name"] == row["owner"])
                package = json.loads(files[owner["manifestPath"]]); package["optionalDependencies"][row["name"]] = row["version"]
                files[owner["manifestPath"]] = json.dumps(package).encode()
        files["docs/releases/v2-candidate.json"] = json.dumps(family).encode()
        plan = self.select(files, family, "release")
        self.check_case("JS-DTS-source-version-candidate", plan, expected, reverse_oracle(self.graph, {key}, "testDeps"))
        self.assertEqual(set(plan["publish"]), expected); self.assertEqual(plan["versionRequired"], [])
        self.assertFalse(plan["publicationBlocked"])

    def test_actual_first_ci_and_release_are_complete_separate_fallbacks(self):
        for mode in ["ci", "release"]:
            plan = chain.plan_chain(mode, self.source, self.files, self.family, [], source_modes=self.modes)
            self.check_case("first-" + mode, plan, set(self.records), set(self.graph))
            self.assertEqual(set(plan["publish"]), set(self.records) if mode == "release" else set())
            self.assertEqual(plan["namespace"], "farm-" + mode + "-v1")


if __name__ == "__main__":
    unittest.main(verbosity=2)
