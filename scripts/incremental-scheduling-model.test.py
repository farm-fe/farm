"""Set-equality oracle: no unrelated package may enter a normal incremental delta."""
# cspell:words importlib
import importlib.util
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("scheduling_fixture", Path(__file__).with_name("incremental-chain-plan.test.py"))
fixtures = importlib.util.module_from_spec(spec); spec.loader.exec_module(fixtures)
chain = fixtures.chain
SDK = {"cargo:" + name for name in chain.SDK_ROOTS}
AB = {"npm:a", "npm:b"}
A_TESTS = {"npm:packages/a", "npm:packages/x", "npm:packages/b", "npm:examples/one"}


def model():
    files, family, source = fixtures.fixture()
    files["packages/d/package.json"] = b'{"name":"d","version":"1.0.0"}'
    files["packages/d/src/index.ts"] = b'export const value = 0;'
    files["packages/x/package.json"] = b'{"name":"x","version":"1.0.0","private":true,"dependencies":{"a":"workspace:*"}}'
    files["packages/x/src/index.ts"] = b'export const bridge = 1;'
    package = json.loads(files["packages/b/package.json"]); package["dependencies"] = {"x": "workspace:*"}
    files["packages/b/package.json"] = json.dumps(package).encode()
    package = json.loads(files["examples/two/package.json"]); package["dependencies"] = {"d": "workspace:*"}
    files["examples/two/package.json"] = json.dumps(package).encode()
    family["npm"]["primary"].append({"name": "d", "version": "1.0.0", "manifestPath": "packages/d/package.json"})
    files["docs/releases/v2-candidate.json"] = json.dumps(family).encode()
    return files, family, source


def select(path=None, mode="release", mutate=None, change=None, source_modes=None):
    files, family, source = model(); ledger, anchor = fixtures.baseline(files, family, mode)
    previous = dict(files)
    if path:
        files[path] = files.get(path, b'') + b'\n// changed input\n'
    if mutate:
        mutate(files, family)
    plan = fixtures.calculate(mode, source, files, family, change or [], ledger, previous, anchor,
                              **({"source_modes": source_modes(files)} if source_modes else {}))
    assert plan["executionAuthorized"] is False and plan["publicationAuthorized"] is False
    return plan


class SchedulingOracle(unittest.TestCase):
    def test_no_change_has_empty_exact_build_test_and_publish_sets(self):
        plan = select()
        self.assertEqual(plan["build"], []); self.assertEqual(plan["testNodes"], [])
        self.assertEqual(plan["publish"], []); self.assertFalse(plan["fullBuild"])

    def test_a_source_selects_only_a_b_and_their_reverse_test_closure(self):
        plan = select("packages/a/src/index.ts")
        self.assertEqual(set(plan["build"]), AB)
        self.assertEqual(set(plan["testNodes"]), A_TESTS)
        self.assertEqual(set(plan["versionRequired"]), AB)
        self.assertEqual(plan["publish"], [])

    def test_b_source_keeps_private_forward_bridge_and_does_not_build_d_or_sdk(self):
        plan = select("packages/b/src/index.ts")
        self.assertEqual(plan["build"], ["npm:b"])
        self.assertEqual(set(plan["testNodes"]), {"npm:packages/b", "npm:examples/one"})
        self.assertEqual(set(plan["buildDependencyClosure"]), AB)
        self.assertEqual(set(plan["buildNodes"]), {"npm:packages/a", "npm:packages/x", "npm:packages/b"})

    def test_private_x_changes_publish_only_b_after_version_preparation(self):
        plan = select("packages/x/src/index.ts")
        self.assertEqual(plan["build"], ["npm:b"])
        self.assertEqual(plan["versionRequired"], ["npm:b"])
        self.assertEqual(set(plan["testNodes"]), A_TESTS - {"npm:packages/a"})
        self.assertEqual(set(plan["buildDependencyClosure"]), AB)

    def test_a_mode_only_change_has_the_same_exact_reverse_impact(self):
        def modes(files):
            result = fixtures.modes(files); result["packages/a/src/index.ts"] = "100755"; return result
        plan = select(source_modes=modes)
        self.assertEqual(set(plan["build"]), AB); self.assertEqual(set(plan["testNodes"]), A_TESTS)

    def test_source_versions_and_generated_candidate_refresh_keep_exact_delta(self):
        def version(files, family):
            for row in family["npm"]["primary"]:
                if row["name"] in ["a", "b"]:
                    row["version"] = "1.0.1"
                    package = json.loads(files[row["manifestPath"]]); package["version"] = row["version"]
                    files[row["manifestPath"]] = json.dumps(package).encode()
            files["docs/releases/v2-candidate.json"] = json.dumps(family).encode()
        plan = select("packages/a/src/index.ts", mutate=version)
        self.assertEqual(set(plan["build"]), AB); self.assertEqual(set(plan["publish"]), AB)
        self.assertEqual(plan["versionRequired"], []); self.assertFalse(plan["publicationBlocked"])
        self.assertEqual(plan["validationRequired"], ["family-metadata-contracts"])

    def test_recipe_rebuild_does_not_invent_product_version_changes(self):
        plan = select("scripts/build.mjs")
        self.assertEqual(set(plan["build"]), SDK | AB | {"npm:d"})
        self.assertEqual(plan["versionRequired"], []); self.assertEqual(plan["publish"], [])
        self.assertEqual(plan["sameVersionRebuildPolicy"], "compare-original-archive-bytes-or-require-new-version")

    def test_unknown_source_and_unresolved_local_alias_have_explicit_full_reasons(self):
        unknown = select("unmapped/new-source.txt", mode="ci")
        self.assertEqual(set(unknown["build"]), SDK | AB | {"npm:d"})
        self.assertIn("unmapped-source-changed", unknown["reasons"])
        def alias(files, family):
            package = json.loads(files["packages/b/package.json"]); package["dependencies"] = {"a": "workspace:another@*"}
            files["packages/b/package.json"] = json.dumps(package).encode()
        plan = select(mode="ci", mutate=alias)
        self.assertEqual(set(plan["build"]), SDK | AB | {"npm:d"})
        self.assertIn("workspace-alias-requires-review", plan["reasons"])

    def test_referenced_duplicate_private_name_is_not_merged_with_public_owner(self):
        def duplicate(files, family):
            files["packages/a-shadow/package.json"] = b'{"name":"a","version":"1.0.0","private":true}'
        plan = select(mode="ci", mutate=duplicate)
        self.assertEqual(set(plan["build"]), SDK | AB | {"npm:d"})
        self.assertIn("ambiguous-dependency", plan["reasons"])
        self.assertIn("npm:packages/a", plan["testNodes"])
        self.assertIn("npm:packages/a-shadow", plan["testNodes"])

    def test_deletion_and_removed_dependency_still_follow_old_reverse_edges(self):
        def remove(files, family):
            del files["packages/a/src/index.ts"]
            package = json.loads(files["packages/x/package.json"]); package["dependencies"] = {}
            files["packages/x/package.json"] = json.dumps(package).encode()
        plan = select(mutate=remove)
        self.assertEqual(set(plan["build"]), AB); self.assertEqual(set(plan["testNodes"]), A_TESTS)

    def test_cross_owner_rename_selects_both_ends_without_sdk(self):
        def rename(files, family):
            files["packages/d/src/moved.ts"] = files.pop("packages/a/src/index.ts")
        plan = select(mutate=rename, change=[{"status": "R100", "paths": ["packages/a/src/index.ts", "packages/d/src/moved.ts"]}])
        self.assertEqual(set(plan["build"]), AB | {"npm:d"})
        self.assertEqual(set(plan["testNodes"]), A_TESTS | {"npm:packages/d", "npm:examples/two"})

    def test_leaf_e2e_does_not_build_or_publish_library_packages(self):
        plan = select("examples/one/e2e.spec.mjs", mode="ci")
        self.assertEqual(plan["build"], []); self.assertEqual(plan["publish"], [])
        self.assertEqual(plan["testNodes"], ["npm:examples/one"]); self.assertFalse(plan["fullBuild"])

    def test_unproven_leaf_ts_remains_conservative_within_its_owner_closure(self):
        plan = select("packages/a/tests/a.spec.ts", mode="ci")
        self.assertEqual(set(plan["build"]), AB)
        self.assertEqual(set(plan["testNodes"]), A_TESTS); self.assertFalse(plan["fullBuild"])

    def test_shared_runner_prerequisites_cover_all_examples_without_extra_public_builds(self):
        for name in ["core", "cli", "runtime", "runtime-plugin-hmr", "runtime-plugin-import-meta"]:
            files, family, source = model(); directory = "packages/" + name
            manifest = directory + "/package.json"; identity = "@farmfe/" + name
            files[manifest] = json.dumps({"name": identity, "version": "1.0.0"}).encode()
            files[directory + "/src/index.ts"] = b'export const value = 1;'
            family["npm"]["primary"].append({"name": identity, "version": "1.0.0", "manifestPath": manifest})
            files["docs/releases/v2-candidate.json"] = json.dumps(family).encode()
            ledger, anchor = fixtures.baseline(files, family, "ci"); previous = dict(files)
            files[directory + "/src/index.ts"] = b'export const value = 2;'
            plan = fixtures.calculate("ci", source, files, family, [], ledger, previous, anchor)
            self.assertEqual(plan["build"], ["npm:" + identity])
            self.assertEqual(set(plan["testNodes"]), {"npm:" + directory, "npm:examples/one", "npm:examples/two"})
            self.assertFalse(plan["fullBuild"])

    def test_first_build_stays_inside_its_chain_and_ci_never_publishes(self):
        files, family, source = model()
        for mode in ["ci", "release"]:
            plan = fixtures.calculate(mode, source, files, family, [])
            self.assertEqual(set(plan["build"]), SDK | AB | {"npm:d"})
            self.assertEqual(plan["namespace"], "farm-" + mode + "-v1")
            self.assertEqual(set(plan["publish"]), SDK | AB | {"npm:d"} if mode == "release" else set())

    def test_abi_cohort_selects_consumers_but_not_independent_create_farm(self):
        files, family, source = model()
        def add_native(directory, name):
            if name == "create-farm":
                family["npm"]["primary"].append({"name": name, "version": "1.0.0", "manifestPath": directory + "/package.json"})
                files[directory + "/package.json"] = json.dumps({"name": name, "version": "1.0.0"}).encode()
            package = json.loads(files[directory + "/package.json"])
            package["napi"] = {"targets": []}; package["optionalDependencies"] = {}
            for abi in chain.impact.TARGETS:
                row = {"name": name + "-" + abi, "version": "1.0.0", "manifestPath": directory + "/npm/" + abi + "/package.json", "owner": name, "platform": abi}
                family["npm"]["payloads"].append(row)
                files[row["manifestPath"]] = json.dumps({"name": row["name"], "version": row["version"]}).encode()
                package["optionalDependencies"][row["name"]] = row["version"]
            files[directory + "/package.json"] = json.dumps(package).encode()
        add_native("packages/a", "a"); add_native("packages/create-farm", "create-farm")
        files["docs/releases/v2-candidate.json"] = json.dumps(family).encode()
        ledger, anchor = fixtures.baseline(files, family)
        previous = dict(files); family["abi"] = "0.5.5"
        files["docs/releases/v2-candidate.json"] = json.dumps(family).encode()
        plan = fixtures.calculate("release", source, files, family, [], ledger, previous, anchor)
        expected = AB | {"npm:a-" + abi for abi in chain.impact.TARGETS}
        self.assertEqual(set(plan["build"]), expected)
        self.assertEqual(set(plan["versionRequired"]), expected)
        self.assertEqual(set(plan["testNodes"]), A_TESTS)
        self.assertNotIn("npm:create-farm", plan["buildDependencyClosure"])
        for mutation in ["missing-payload", "optional", "version"]:
            bad_files = dict(files); bad_family = json.loads(json.dumps(family))
            row = bad_family["npm"]["payloads"][0]
            if mutation == "missing-payload":
                bad_family["npm"]["payloads"].pop(0)
            elif mutation == "optional":
                package = json.loads(bad_files["packages/a/package.json"])
                del package["optionalDependencies"][row["name"]]
                bad_files["packages/a/package.json"] = json.dumps(package).encode()
            else:
                package = json.loads(bad_files[row["manifestPath"]]); package["version"] = "1.0.1"
                bad_files[row["manifestPath"]] = json.dumps(package).encode()
            bad_files["docs/releases/v2-candidate.json"] = json.dumps(bad_family).encode()
            with self.assertRaises(ValueError):
                fixtures.calculate("release", source, bad_files, bad_family, [], ledger, previous, anchor)

    @unittest.skip("Not implemented: real publication lock and concurrent candidate execution")
    def test_real_concurrent_publication_lock(self):
        pass

    @unittest.skip("Not implemented: new incremental release executor and durable partial-upload recovery")
    def test_real_incremental_release_recovery(self):
        pass


if __name__ == "__main__":
    unittest.main(verbosity=2)
