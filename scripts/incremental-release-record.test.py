"""Actual-stage result recording contracts with a read-only fake registry."""
# cspell:words importlib
import copy
import importlib.util
from pathlib import Path
import unittest
from unittest import mock

spec = importlib.util.spec_from_file_location("release_record_contract", Path(__file__).with_name("incremental-release-record.py"))
module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)


class Contracts(unittest.TestCase):
    def fixture(self):
        manifest = {"sourceTree": "b" * 40, "planHash": "c" * 64, "packages": [{"kind": "npm", "name": "a", "version": "2.0.0", "archiveSha256": "d" * 64}]}
        report = {"status": "passed", "publicRegistry": True, "workspacePatches": False, "cleanupConfirmed": True,
                  "stages": ["npm-install", "esm-cjs", "types", "docs-build", "sdk"]}
        reports = {platform: copy.deepcopy(report) for platform in ["ubuntu-24.04", "macos-latest", "windows-latest"]}
        registry = mock.Mock(); registry.verify.return_value = {"npm:a": "d" * 64}
        registry.json.return_value = {"name": "a", "version": "2.0.0"}
        return manifest, reports, registry

    def test_public_proof_requires_actual_stages_all_platforms_registry_bytes_and_latest(self):
        manifest, reports, registry = self.fixture()
        result = module.public_record(manifest, reports, {"id": 12}, registry)
        self.assertEqual(result["packages"], {"npm:a": "d" * 64})
        self.assertTrue(result["latestTargets"]); registry.verify.assert_called_once()

    def test_missing_platform_or_unconfirmed_cleanup_blocks_proof(self):
        manifest, reports, registry = self.fixture(); reports.pop("macos-latest")
        with self.assertRaisesRegex(ValueError, "three public"): module.public_record(manifest, reports, {"id": 12}, registry)
        manifest, reports, registry = self.fixture(); reports["windows-latest"]["cleanupConfirmed"] = False
        with self.assertRaisesRegex(ValueError, "cleanup failed"): module.public_record(manifest, reports, {"id": 12}, registry)
        registry.verify.assert_not_called()

    def test_missing_sdk_or_different_registry_byte_digest_cannot_be_reported_passed(self):
        manifest, reports, registry = self.fixture(); reports["ubuntu-24.04"]["stages"].remove("sdk")
        with self.assertRaisesRegex(ValueError, "stages missing"): module.public_record(manifest, reports, {"id": 12}, registry)
        manifest, reports, registry = self.fixture(); registry.verify.return_value = {"npm:a": "e" * 64}
        with self.assertRaisesRegex(ValueError, "bytes differ"): module.public_record(manifest, reports, {"id": 12}, registry)

    def test_newer_latest_is_preserved_but_older_or_other_package_rejected(self):
        manifest, reports, registry = self.fixture(); registry.json.return_value = {"name": "a", "version": "2.1.0"}
        self.assertTrue(module.public_record(manifest, reports, {"id": 12}, registry)["latestTargets"])
        for value in [{"name": "a", "version": "1.9.0"}, {"name": "other", "version": "2.0.0"}]:
            registry.json.return_value = value
            with self.assertRaises(ValueError): module.public_record(manifest, reports, {"id": 12}, registry)


if __name__ == "__main__":
    unittest.main(verbosity=2)
