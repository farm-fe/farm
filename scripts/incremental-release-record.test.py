"""Actual-stage result recording contracts with a read-only fake registry."""
# cspell:words importlib
import base64
import io
import json
import urllib.parse
import copy
import importlib.util
import hashlib
import tempfile
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
        registry = mock.MagicMock(); registry.verify.return_value = {"npm:a": "d" * 64}
        registry.json.return_value = {"name": "a", "version": "2.0.0"}
        return manifest, reports, registry

    def test_byte_integrity_does_not_claim_or_require_tests(self):
        manifest,_,registry=self.fixture()
        manifest.update(sourceSHA='a'*40,chain='release',namespace='farm-release-v1',releaseValidation='archive-integrity-v1')
        body=b'original sealed archive fixture'
        row=manifest['packages'][0]
        row.update(archive='a.tgz',archiveBytes=len(body),archiveSha256=hashlib.sha256(body).hexdigest())
        registry.verify.return_value={'npm:a':row['archiveSha256']}
        with tempfile.TemporaryDirectory() as directory:
            archives=Path(directory)/'archives';archives.mkdir();file=archives/'a.tgz';file.write_bytes(body)
            self.assertIsNone(module.check_archives(manifest,directory,'a'*40))
            proof=module.registry_record(manifest,{'id':12},registry)
            self.assertTrue(module.entry.public_integrity_verified(proof,manifest))
            self.assertFalse(proof['registryInstall']);self.assertFalse(proof['nativeABI']);self.assertFalse(proof['testsExecuted'])
            self.assertNotIn('scratchRemoved',proof);self.assertNotIn('cleanupConfirmed',proof)
            for field,value in [('testsExecuted',True),('registryBytesVerified',False),('registryInstall',True),('latestTargets',False)]:
                self.assertFalse(module.entry.public_integrity_verified({**proof,field:value},manifest))
            with self.assertRaisesRegex(ValueError,'source differs'):module.check_archives(manifest,directory,'f'*40)
            registry.verify.return_value={'npm:a':'0'*64}
            with self.assertRaisesRegex(ValueError,'registry bytes'):module.registry_record(manifest,{'id':12},registry)
            file.write_bytes(body.replace(b'original',b'modified'))
            with self.assertRaisesRegex(ValueError,'bytes differ'):module.check_archives(manifest,directory,'a'*40)
            file.unlink()
            with self.assertRaises(FileNotFoundError):module.check_archives(manifest,directory,'a'*40)

    def test_frozen_manifest_alone_selects_old_or_new_proof_semantics(self):
        legacy,reports,registry=self.fixture();current={**legacy,'releaseValidation':'archive-integrity-v1'}
        old=module.public_record(legacy,reports,{'id':12},registry)
        new=module.registry_record(current,{'id':12},registry)
        self.assertTrue(module.entry.public_integrity_verified(old,legacy))
        self.assertTrue(module.entry.public_integrity_verified(new,current))
        self.assertFalse(module.entry.public_integrity_verified(new,legacy))
        self.assertFalse(module.entry.public_integrity_verified(old,current))
        with self.assertRaisesRegex(ValueError,'policy'):module.public_record(current,reports,{'id':12},registry)
        with self.assertRaisesRegex(ValueError,'policy'):module.registry_record(legacy,{'id':12},registry)
        reports['windows-latest']['kind']='release-byte-integrity-v1'
        with self.assertRaisesRegex(ValueError,'byte-only'):module.public_record(legacy,reports,{'id':12},registry)

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


class InventoryBudgetContracts(unittest.TestCase):
    def fixture(self, seconds=1):
        now = [1000.0]; calls = []
        body = b'frozen production-algorithm test bytes'
        rows = [{'kind': 'npm', 'name': 'package-' + str(i), 'version': '2.0.0',
                 'archiveSha256': hashlib.sha256(body).hexdigest()} for i in range(297)]
        class Opener:
            def open(self, request, timeout):
                calls.append(request.full_url); now[0] += seconds
                name, selector = urllib.parse.urlsplit(request.full_url).path.strip('/').split('/')
                if selector == 'archive.tgz': value = body
                else:
                    value = json.dumps({'name': name, 'version': '2.0.0', 'dist': {
                        'tarball': 'https://registry.npmjs.org/' + name + '/archive.tgz',
                        'integrity': 'sha512-' + base64.b64encode(hashlib.sha512(body).digest()).decode()}}).encode()
                return io.BytesIO(value)
        client = module.entry.evidence.registry_module.RegistryReadOnly({'GITHUB_ACTIONS': 'true'}, Opener(), lambda: now[0])
        manifest = {'releaseValidation': 'archive-integrity-v1', 'sourceTree': 'b' * 40, 'planHash': 'c' * 64, 'packages': rows}
        now[0] += 1000  # External source/lock work must not age the reader.
        return manifest, client, now, calls

    def test_full_public_proof_covers_archives_and_latest_with_one_batch_deadline(self):
        manifest, client, now, calls = self.fixture()
        result = module.registry_record(manifest, {'id': 7}, client)
        self.assertEqual(len(result['packages']), 297)
        self.assertEqual(len(calls), 891)
        self.assertEqual(now[0], 2891)
        self.assertTrue(result['registryBytesVerified'])
        self.assertFalse(result['testsExecuted'])

    def test_latest_phase_cannot_restart_exhausted_public_batch(self):
        manifest, client, _, calls = self.fixture(seconds=2)
        with self.assertRaisesRegex(ValueError, 'deadline'):
            module.registry_record(manifest, {'id': 7}, client)
        self.assertGreater(len(calls), 594)  # archive loop passed; latest uses same budget
        self.assertLess(len(calls), 891)

    def test_finalize_style_full_inventory_and_recovery_retain_byte_accounting(self):
        manifest, client, now, calls = self.fixture(seconds=2)
        records = {'npm:' + row['name']: row for row in manifest['packages']}
        expected = {key: row['archiveSha256'] for key, row in records.items()}
        self.assertEqual(client.verify(records), expected)
        count = client.total_bytes
        now[0] += 1000
        self.assertEqual(client.verify(records), expected)
        self.assertEqual(client.total_bytes, 2 * count)
        self.assertEqual(len(calls), 1188)


if __name__ == "__main__":
    unittest.main(verbosity=2)
