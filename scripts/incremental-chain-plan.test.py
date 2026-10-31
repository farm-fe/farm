"""Independent chain selection contracts; execution belongs to remote CI."""
# cspell:words importlib
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("chain", Path(__file__).with_name("incremental-chain-plan.py"))
chain = importlib.util.module_from_spec(spec)
spec.loader.exec_module(chain)


def fixture():
    files = {"Cargo.toml": b'[workspace]\nmembers=["crates/*", "rust-ecosystem/*"]\n',
             "pnpm-workspace.yaml": b'packages:\n  - packages/*\n  - examples/*\n',
             "crates/core/Cargo.toml": b'[package]\nname="farmfe_core"\nversion="1.0.0"\n',
             "crates/core/src/lib.rs": b'pub fn value() {}'}
    for directory, name in [("crates/compiler", "farmfe_compiler"), ("crates/toolkit", "farmfe_toolkit"), ("rust-ecosystem/glob", "farmfe_ecosystem_glob")]:
        files[directory + "/Cargo.toml"] = ('[package]\nname="' + name + '"\nversion="1.0.0"\n').encode()
    primary = []
    for directory, name, dependencies in [("packages/a", "a", {}), ("packages/b", "b", {"a": "workspace:*"}),
                                           ("examples/one", "demo", {"b": "workspace:*"}), ("examples/two", "demo", {})]:
        manifest = directory + "/package.json"
        files[manifest] = json.dumps({"name": name, "version": "1.0.0", "dependencies": dependencies}).encode()
        files[directory + "/src/index.ts"] = b'export const value = 1;'
        if directory.startswith("packages/"):
            primary.append({"name": name, "version": "1.0.0", "manifestPath": manifest})
    family = {"abi": "0.5.4", "cargo": [{"name": name, "version": "1.0.0", "manifestPath": directory + "/Cargo.toml"}
              for directory, name in [("crates/core", "farmfe_core"), ("crates/compiler", "farmfe_compiler"),
                                      ("crates/toolkit", "farmfe_toolkit"), ("rust-ecosystem/glob", "farmfe_ecosystem_glob")]],
              "npm": {"primary": primary, "payloads": []}}
    source = {"sha": "a" * 40, "tree": "b" * 40}
    return files, family, source


def modes(files):
    return {name: "100644" for name in files}


def calculate(mode, source, files, family, changes, ledger=None, previous=None, anchor=None, **options):
    options.setdefault("source_modes", modes(files))
    if previous is not None:
        options.setdefault("baseline_modes", modes(previous))
    return chain.plan_chain(mode, source, files, family, changes, ledger, previous, anchor, **options)


def baseline(files, family, mode="release"):
    ledger = {"schemaVersion": 1, "chain": mode, "profile": mode, "policy": chain.POLICY, "status": "success",
              "repositoryId": 1, "runId": 2, "runAttempt": 1, "sourceSHA": "c" * 40, "sourceTree": "d" * 40,
              "registryVerified": True, "abi": family["abi"],
              "sourceSnapshotHash": chain.digest(chain.snapshot_records(files, modes(files))),
              "packages": {key: {"identity": key, "version": row["version"], "archiveSha256": "e" * 64}
                           for key, row in chain.family_records(family).items()}}
    anchor = {key: ledger[key] for key in ["chain", "repositoryId", "runId", "runAttempt", "sourceSHA", "sourceTree"]}
    anchor.update({"verified": True, "ledgerSha256": chain.digest(ledger)})
    anchor["registryChecksums"] = {key: row["archiveSha256"] for key, row in ledger["packages"].items()}
    return ledger, anchor


class ChainContracts(unittest.TestCase):
    def test_first_release_rebuilds_full_family_in_its_own_namespace(self):
        files, family, source = fixture()
        plan = calculate("release", source, files, family, [])
        self.assertTrue(plan["fullBuild"])
        self.assertEqual(plan["build"], sorted(chain.family_records(family)))
        self.assertEqual(plan["publish"], plan["build"])
        self.assertEqual(plan["profile"], "release")
        self.assertFalse(plan["crossChainReuseAllowed"])

    def test_chain_mismatch_is_rejected_even_with_a_matching_ledger_hash(self):
        files, family, source = fixture(); ledger, anchor = baseline(files, family, "ci")
        with self.assertRaisesRegex(ValueError, "chain differs"):
            calculate("release", source, files, family, [], ledger, files, anchor)

    def test_reverse_dependencies_require_versions_but_private_examples_do_not_publish(self):
        files, family, source = fixture(); ledger, anchor = baseline(files, family)
        plan = calculate("release", source, files, family,
                                [{"status": "M", "paths": ["packages/a/src/index.ts"]}], ledger, files, anchor)
        self.assertEqual(plan["build"], ["npm:a", "npm:b"])
        self.assertEqual(plan["versionRequired"], ["npm:a", "npm:b"])
        self.assertTrue(plan["publicationBlocked"])
        self.assertIn("npm:examples/one", plan["testNodes"])
        self.assertNotIn("npm:examples/two", plan["testNodes"])

    def test_changed_versions_form_publish_delta_without_republishing_core(self):
        files, family, source = fixture(); ledger, anchor = baseline(files, family)
        family = json.loads(json.dumps(family))
        previous_files = dict(files)
        for row in family["npm"]["primary"]:
            row["version"] = "1.0.1"
            manifest = json.loads(files[row["manifestPath"]]); manifest["version"] = row["version"]
            files[row["manifestPath"]] = json.dumps(manifest).encode()
        plan = calculate("release", source, files, family,
                                [{"status": "M", "paths": ["packages/a/src/index.ts"]}], ledger, previous_files, anchor)
        self.assertEqual(plan["publish"], ["npm:a", "npm:b"])
        self.assertEqual(plan["unchanged"], sorted("cargo:" + row["name"] for row in family["cargo"]))
        self.assertFalse(plan["publicationBlocked"])

    def test_recipe_rebuild_is_distinct_from_forcing_new_package_versions(self):
        files, family, source = fixture(); ledger, anchor = baseline(files, family)
        plan = calculate("release", source, files, family,
                                [{"status": "M", "paths": ["scripts/build.mjs"]}], ledger, files, anchor)
        self.assertTrue(plan["fullBuild"])
        self.assertEqual(plan["versionRequired"], [])
        self.assertEqual(plan["publish"], [])

    def test_ci_never_authorizes_publication(self):
        files, family, source = fixture(); ledger, anchor = baseline(files, family, "ci")
        plan = calculate("ci", source, files, family, [], ledger, files, anchor)
        self.assertEqual(plan["build"], [])
        self.assertEqual(plan["publish"], [])
        self.assertTrue(plan["publicationBlocked"])
        self.assertEqual(plan["namespace"], "farm-ci-v1")

    def test_an_omitted_changed_path_cannot_hide_product_changes(self):
        files, family, source = fixture(); ledger, anchor = baseline(files, family)
        previous_files = dict(files); files["packages/a/src/index.ts"] = b'export const value = 2;'
        plan = calculate("release", source, files, family, [], ledger, previous_files, anchor)
        self.assertEqual(plan["versionRequired"], ["npm:a", "npm:b"])

    def test_inventory_versions_must_match_real_manifests(self):
        files, family, source = fixture(); family["npm"]["primary"][0]["version"] = "1.0.1"
        with self.assertRaisesRegex(ValueError, "actual source manifest"):
            calculate("release", source, files, family, [])

    def test_abi_change_requires_all_farm_native_owners_and_payload_versions(self):
        files, family, source = fixture()
        package = json.loads(files["packages/a/package.json"])
        package["napi"] = {"targets": []}; package["optionalDependencies"] = {}
        for platform in chain.impact.TARGETS:
            payload = {"name": "a-" + platform, "version": "1.0.0", "owner": "a", "platform": platform,
                       "manifestPath": "packages/a/npm/" + platform + "/package.json"}
            family["npm"]["payloads"].append(payload)
            files[payload["manifestPath"]] = json.dumps({"name": payload["name"], "version": payload["version"]}).encode()
            package["optionalDependencies"][payload["name"]] = payload["version"]
        files["packages/a/package.json"] = json.dumps(package).encode()
        ledger, anchor = baseline(files, family)
        family["abi"] = "0.5.5"
        plan = calculate("release", source, files, family, [], ledger, files, anchor)
        self.assertEqual(plan["versionRequired"], sorted(["npm:a", "npm:b", *["npm:a-" + target for target in chain.impact.TARGETS]]))
        self.assertTrue(plan["publicationBlocked"])

    def test_delete_preserves_old_reverse_dependency_edges(self):
        files, family, source = fixture(); ledger, anchor = baseline(files, family)
        previous_files = dict(files); del files["packages/a/src/index.ts"]
        plan = calculate("release", source, files, family, [], ledger, previous_files, anchor)
        self.assertEqual(plan["versionRequired"], ["npm:a", "npm:b"])

    def test_release_requires_each_original_registry_checksum(self):
        files, family, source = fixture(); ledger, anchor = baseline(files, family)
        del anchor["registryChecksums"]["npm:a"]
        with self.assertRaisesRegex(ValueError, "registry checksum"):
            calculate("release", source, files, family, [], ledger, files, anchor)

    def test_baseline_cannot_be_self_authenticated_or_partial(self):
        files, family, source = fixture(); ledger, anchor = baseline(files, family)
        with self.assertRaisesRegex(ValueError, "verified anchor"):
            calculate("release", source, files, family, [], ledger, files)
        ledger["status"] = "partial"; anchor["ledgerSha256"] = chain.digest(ledger)
        with self.assertRaisesRegex(ValueError, "successful"):
            calculate("release", source, files, family, [], ledger, files, anchor)

    def test_mode_only_change_affects_source_hash_and_build_selection(self):
        files, family, source = fixture(); ledger, anchor = baseline(files, family)
        changed_modes = modes(files); changed_modes["packages/a/src/index.ts"] = "100755"
        plan = calculate("release", source, files, family, [], ledger, files, anchor, source_modes=changed_modes)
        self.assertEqual(plan["versionRequired"], ["npm:a", "npm:b"])
        self.assertNotEqual(plan["sourceSnapshotHash"], ledger["sourceSnapshotHash"])

    def test_missing_mode_evidence_is_rejected(self):
        files, family, source = fixture()
        with self.assertRaisesRegex(ValueError, "file mode"):
            calculate("release", source, files, family, [], source_modes={})

    def test_family_cannot_omit_a_manifest_or_sdk_dependency(self):
        for group in ["npm", "cargo"]:
            files, family, source = fixture()
            (family["npm"]["primary"] if group == "npm" else family["cargo"]).pop()
            with self.assertRaisesRegex(ValueError, "full family"):
                calculate("release", source, files, family, [])

    def test_baseline_cannot_omit_an_unchanged_package(self):
        files, family, source = fixture(); ledger, anchor = baseline(files, family)
        del ledger["packages"]["npm:b"]; anchor["ledgerSha256"] = chain.digest(ledger)
        with self.assertRaisesRegex(ValueError, "full family coverage"):
            calculate("release", source, files, family, [], ledger, files, anchor)

    def test_unresolved_local_protocols_require_a_full_build(self):
        for declaration in ["workspace:*", "workspace:../missing", "file:../missing", "link:../missing", "npm:missing@1.0.0"]:
            files, family, source = fixture()
            manifest = json.loads(files["packages/a/package.json"])
            manifest["dependencies"]["missing"] = declaration
            files["packages/a/package.json"] = json.dumps(manifest).encode()
            ledger, anchor = baseline(files, family, "ci")
            plan = calculate("ci", source, files, family, [], ledger, files, anchor)
            self.assertTrue(plan["fullBuild"])
            self.assertTrue(set(plan["reasons"]) & {"missing-local-dependency", "alias-dependency-requires-review"})

    def test_forward_build_dependencies_are_separate_from_changed_packages(self):
        files, family, source = fixture(); ledger, anchor = baseline(files, family)
        plan = calculate("release", source, files, family,
                         [{"status": "M", "paths": ["packages/b/src/index.ts"]}], ledger, files, anchor)
        self.assertEqual(plan["build"], ["npm:b"])
        self.assertEqual(plan["buildDependencyClosure"], ["npm:a", "npm:b"])
        self.assertFalse(plan["executionAuthorized"])
        self.assertFalse(plan["publicationAuthorized"])

    def test_lockfile_full_fallback_does_not_require_unrelated_excluded_copy_scope(self):
        files, family, source = fixture()
        files["pnpm-workspace.yaml"] += b'  - js-plugins/*\n'
        files["js-plugins/copy/package.json"] = b'{"name":"@farmfe/js-plugin-copy","version":"0.0.4"}'
        ledger, anchor = baseline(files, family, "ci")
        previous = dict(files); files["Cargo.lock"] = b'version = 4\n'
        plan = calculate("ci", source, files, family, [], ledger, previous, anchor)
        self.assertTrue(plan["fullBuild"])
        self.assertEqual(plan["build"], sorted(chain.family_records(family)))
        with self.assertRaisesRegex(ValueError, "scope review"):
            calculate("ci", source, files, family,
                      [{"status": "M", "paths": ["js-plugins/copy/package.json"]}], ledger, previous, anchor)

    def test_forward_closure_crosses_a_non_published_build_bridge(self):
        files, family, source = fixture()
        files["packages/bridge/package.json"] = b'{"name":"bridge","version":"1.0.0","private":true,"dependencies":{"a":"workspace:*"}}'
        manifest = json.loads(files["packages/b/package.json"])
        manifest["dependencies"] = {"bridge": "workspace:*"}
        files["packages/b/package.json"] = json.dumps(manifest).encode()
        ledger, anchor = baseline(files, family)
        plan = calculate("release", source, files, family,
                         [{"status": "M", "paths": ["packages/b/src/index.ts"]}], ledger, files, anchor)
        self.assertEqual(plan["build"], ["npm:b"])
        self.assertEqual(plan["buildDependencyClosure"], ["npm:a", "npm:b"])
        self.assertIn("npm:packages/bridge", plan["buildNodes"])

    def test_workspace_alias_does_not_silently_resolve_to_its_key(self):
        files, family, source = fixture()
        manifest = json.loads(files["packages/b/package.json"])
        manifest["dependencies"] = {"a": "workspace:other@*"}
        files["packages/b/package.json"] = json.dumps(manifest).encode()
        ledger, anchor = baseline(files, family, "ci")
        plan = calculate("ci", source, files, family, [], ledger, files, anchor)
        self.assertTrue(plan["fullBuild"])
        self.assertIn("workspace-alias-requires-review", plan["reasons"])

    def test_source_manifest_and_candidate_refresh_keeps_a_b_publication_delta(self):
        files, family, source = fixture()
        files["docs/releases/v2-candidate.json"] = json.dumps(family).encode()
        ledger, anchor = baseline(files, family)
        previous = dict(files); family = copy_family = json.loads(json.dumps(family))
        files["packages/a/src/index.ts"] = b'export const value = 2;'
        for row in copy_family["npm"]["primary"]:
            row["version"] = "1.0.1"
            manifest = json.loads(files[row["manifestPath"]]); manifest["version"] = row["version"]
            files[row["manifestPath"]] = json.dumps(manifest).encode()
        files["docs/releases/v2-candidate.json"] = json.dumps(copy_family).encode()
        plan = calculate("release", source, files, family, [], ledger, previous, anchor)
        self.assertEqual(plan["publish"], ["npm:a", "npm:b"])
        self.assertEqual(plan["versionRequired"], [])
        self.assertEqual(plan["build"], ["npm:a", "npm:b"])
        self.assertFalse(plan["publicationBlocked"])

    def test_candidate_metadata_cannot_hide_or_change_a_validated_family_record(self):
        files, family, source = fixture()
        broken = json.loads(json.dumps(family)); broken["npm"]["primary"].pop()
        files["docs/releases/v2-candidate.json"] = json.dumps(broken).encode()
        with self.assertRaisesRegex(ValueError, "metadata differs"):
            calculate("release", source, files, family, [])

    def test_valid_metadata_only_refresh_does_not_invent_product_changes(self):
        files, family, source = fixture(); files["docs/releases/v2-candidate.json"] = json.dumps(family).encode()
        ledger, anchor = baseline(files, family); previous = dict(files)
        refreshed = {**family, "status": "candidate-metadata-refreshed"}
        files["docs/releases/v2-candidate.json"] = json.dumps(refreshed).encode()
        plan = calculate("release", source, files, family, [], ledger, previous, anchor)
        self.assertEqual(plan["build"], []); self.assertEqual(plan["versionRequired"], [])
        self.assertEqual(plan["publish"], []); self.assertFalse(plan["fullBuild"])
        self.assertEqual(plan["validationRequired"], ["family-metadata-contracts"])


if __name__ == "__main__":
    unittest.main()
