"""Registry host transfer and materialization boundaries; no network or native builds."""
# cspell:words importlib userconfig globalconfig npmjs glibc
import base64
import copy
import gzip
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import unittest
from unittest import mock

spec = importlib.util.spec_from_file_location('registry_host_tests', Path(__file__).with_name('release-registry-host-tool.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def elf():
    body = bytearray(64)
    body[:6] = b'\x7fELF\x02\x01'
    body[16:18] = (3).to_bytes(2, 'little')
    body[18:20] = (62).to_bytes(2, 'little')
    return bytes(body)


def manifest(**changes):
    return {'name': module.NAME, 'version': '2.0.0', 'main': module.NATIVE,
            'os': ['linux'], 'cpu': ['x64'], 'libc': ['glibc'], **changes}


def member(name, body=b'', kind=tarfile.REGTYPE, linkname=''):
    item = tarfile.TarInfo(name)
    item.size = len(body)
    item.type = kind
    item.linkname = linkname
    return item, body


def archive(value=None, binary=None, extra=(), raw=None):
    raw = raw if raw is not None else json.dumps(manifest() if value is None else value).encode()
    entries = [member('package/package.json', raw),
               member('package/' + module.NATIVE, elf() if binary is None else binary),
               member('package/LICENSE', b'MIT')]
    output = io.BytesIO()
    with tarfile.open(fileobj=output, mode='w', format=tarfile.PAX_FORMAT) as tar:
        for item, body in [*entries, *extra]:
            tar.addfile(item, io.BytesIO(body))
    return gzip.compress(output.getvalue(), mtime=0)


def reseal(selection):
    selection['planHash'] = module.digest({key: value for key, value in selection.items() if key != 'planHash'})
    return selection


def fixture(body=None):
    body = archive() if body is None else body
    row = {'kind': 'npm', 'name': module.NAME, 'version': '2.0.0', 'owner': '@farmfe/core',
           'platform': module.PLATFORM, 'manifestPath': module.MANIFEST, 'status': 'published',
           'evidenceKind': 'registry-metadata',
           'integrity': 'sha512-' + base64.b64encode(hashlib.sha512(body).digest()).decode(),
           'archiveSha256': hashlib.sha256(body).hexdigest(), 'archiveBytes': len(body),
           'tarballURL': 'https://registry.npmjs.org/@farmfe/core-linux-x64-gnu/-/core-linux-x64-gnu-2.0.0.tgz',
           'metadata': {}}
    selection = reseal({'selectionPolicy': 'registry-exact-version-v2', 'chain': 'release', 'profile': 'release',
                       'sourceSHA': 'a' * 40, 'sourceTree': 'b' * 40,
                       'build': [], 'publish': [], 'nativeBuildCells': [], 'preparationNativeCells': [],
                       'registryRetained': {module.KEY: row},
                       'registryBuildInputs': {module.KEY: {**row, 'origin': 'registry-present'}}})
    metadata = {'name': module.NAME, 'version': '2.0.0',
                'dist': {'tarball': row['tarballURL'], 'integrity': row['integrity']}}
    source = {'sha': 'a' * 40, 'tree': 'b' * 40, 'manifest': manifest(),
              'runtime': {'src/modules/module-helper.ts': b'fixture'}}
    routes = {'https://registry.npmjs.org/%40farmfe%2Fcore-linux-x64-gnu/2.0.0': json.dumps(metadata).encode(),
              row['tarballURL']: body}
    return selection, source, routes


def proof(**changes):
    return {'node': module.NODE_VERSION, 'napi': '10', 'platform': 'linux', 'arch': 'x64', 'libc': 'glibc',
            'requiredExports': module.EXPORTS, 'resolver': True, 'compile': True,
            'tsTransformed': True, 'outputExecuted': True, 'outputBytes': 475,
            'outputSha256': 'c' * 64, **changes}


class Contracts(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix='farm-host-tool-test-')
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        (self.root / 'packages/core/binding').mkdir(parents=True)
        self.destination = self.root / module.DESTINATION
        self.output = self.root / module.OUTPUT
        self.selection, self.source, self.routes = fixture()
        self.read = mock.Mock(side_effect=lambda url, limit: self.routes[url])
        self.smoke = mock.Mock(return_value=proof())

    def materialize(self):
        return module.materialize(self.selection, self.root, self.destination, self.output,
                                  '/trusted/node', source_reader=lambda root: self.source,
                                  read=self.read, smoke=self.smoke)

    def test_success_restores_only_native_and_reports_registry_not_source_provenance(self):
        original = copy.deepcopy(self.selection)
        result = self.materialize()
        self.assertEqual(self.destination.read_bytes(), elf())
        self.assertEqual(json.loads(self.output.read_text()), result)
        self.assertEqual(result['origin'], 'registry-present')
        self.assertFalse(result['sourceNativeArtifact'])
        self.assertFalse(result['publicationCandidate'])
        self.assertNotIn('abi', result)
        self.assertEqual(result['smoke']['node'], 'v22.20.0')
        self.assertEqual(result['binarySha256'], hashlib.sha256(elf()).hexdigest())
        self.assertEqual(original, self.selection)
        self.assertEqual(sorted(str(file.relative_to(self.root)) for file in self.root.rglob('*') if file.is_file()),
                         [module.DESTINATION, module.OUTPUT])
        self.assertEqual(self.destination.stat().st_mode & 0o777, 0o444)
        args = self.smoke.call_args.args
        self.assertEqual(args[1].name, module.NATIVE)
        self.assertFalse(args[3].exists(), 'Owned probe scratch must be cleaned')

    def test_wrong_policy_source_hash_or_selected_record_rejects_before_network(self):
        cases = [lambda s: s.update(selectionPolicy='registry-exact-version-v1'),
                 lambda s: s.update(sourceSHA='f' * 40),
                 lambda s: s.update(sourceTree='f' * 40),
                 lambda s: s.update(profile='ci'),
                 lambda s: s.update(registryBuildInputs={}),
                 lambda s: s['registryBuildInputs'][module.KEY].update(origin='source-build'),
                 lambda s: s['registryBuildInputs'][module.KEY].update(version='3.0.0'),
                 lambda s: s.update(build=[module.KEY]),
                 lambda s: s.update(publish=[module.KEY]),
                 lambda s: s.update(preparationNativeCells=[{'package': module.KEY}])]
        for mutate in cases:
            with self.subTest(mutation=mutate):
                self.selection = fixture()[0]
                mutate(self.selection)
                reseal(self.selection)
                with self.assertRaises(ValueError):
                    self.materialize()
                self.read.assert_not_called()
        self.selection = fixture()[0]
        self.selection['planHash'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'digest'):
            self.materialize()

    def test_metadata_only_selection_measures_archive_only_for_host_execution(self):
        for section in ['registryRetained', 'registryBuildInputs']:
            self.selection[section][module.KEY].pop('archiveSha256')
            self.selection[section][module.KEY].pop('archiveBytes')
        reseal(self.selection)
        result = self.materialize()
        body = self.routes[result['tarballURL']]
        self.assertEqual(result['archiveSha256'], hashlib.sha256(body).hexdigest())
        self.assertEqual(result['archiveBytes'], len(body))
        self.assertEqual(result['evidenceKind'], 'registry-archive')
        self.assertTrue(result['registryBytesVerified'])
        self.assertNotIn('archiveSha256', self.selection['registryBuildInputs'][module.KEY])
        self.assertEqual(self.read.call_count, 2)

    def test_metadata_only_selection_still_rejects_tampered_archive_by_sha512(self):
        for section in ['registryRetained', 'registryBuildInputs']:
            self.selection[section][module.KEY].pop('archiveSha256')
            self.selection[section][module.KEY].pop('archiveBytes')
        reseal(self.selection)
        url = self.selection['registryBuildInputs'][module.KEY]['tarballURL']
        self.routes[url] = self.routes[url] + b'tampered'
        with self.assertRaisesRegex(ValueError, 'transfer integrity'):
            self.materialize()
        self.smoke.assert_not_called()
        self.assertFalse(self.destination.exists())

    def test_registry_identity_platform_version_and_integrity_are_exact(self):
        for field, value in [('owner', 'other'), ('platform', 'darwin-arm64'), ('kind', 'cargo'),
                             ('name', 'other'), ('manifestPath', '../outside'), ('status', 'missing'),
                             ('version', '2.0.1'), ('archiveSha256', 'bad'),
                             ('integrity', 'sha512-YQ=='), ('archiveBytes', True),
                             ('tarballURL', 'https://registry.npmjs.org/a?token=secret')]:
            with self.subTest(field=field):
                self.selection = fixture()[0]
                for section in ['registryRetained', 'registryBuildInputs']:
                    self.selection[section][module.KEY][field] = value
                reseal(self.selection)
                with self.assertRaises(ValueError):
                    self.materialize()
                self.read.assert_not_called()

    def test_fresh_metadata_identity_url_integrity_and_transfer_pin_rechecked(self):
        metadata_url = next(iter(self.routes))
        original = self.routes[metadata_url]
        mutations = [lambda value: value.update(name='other'), lambda value: value.update(version='2.0.1'),
                     lambda value: value['dist'].update(tarball='https://evil.test/a.tgz'),
                     lambda value: value['dist'].update(tarball='https://registry.npmjs.org/other.tgz'),
                     lambda value: value['dist'].update(integrity='sha512-other')]
        for mutate in mutations:
            with self.subTest(mutation=mutate):
                value = json.loads(original)
                mutate(value)
                self.routes[metadata_url] = json.dumps(value).encode()
                with self.assertRaises(ValueError):
                    self.materialize()
                self.smoke.assert_not_called()
                self.assertFalse(self.destination.exists())
        self.routes[metadata_url] = original
        archive_url = self.selection['registryRetained'][module.KEY]['tarballURL']
        body = self.routes[archive_url]
        self.routes[archive_url] = body[:-1] + bytes([body[-1] ^ 1])
        with self.assertRaisesRegex(ValueError, 'integrity'):
            self.materialize()

    def test_no_rebuild_or_partial_destination_on_network_or_smoke_failure(self):
        self.read.side_effect = TimeoutError('fixture unavailable')
        with self.assertRaises(TimeoutError):
            self.materialize()
        self.smoke.assert_not_called()
        self.assertFalse(self.destination.exists())
        self.assertFalse(self.output.exists())
        self.read.side_effect = lambda url, limit: self.routes[url]
        self.smoke.side_effect = ValueError('fixture API mismatch')
        with self.assertRaisesRegex(ValueError, 'API mismatch'):
            self.materialize()
        self.assertFalse(self.destination.exists())
        self.assertFalse(self.output.exists())

    def test_archive_traversal_links_duplicates_case_collisions_and_extra_code_reject(self):
        entries = [member('package/../outside', b'no'),
                   member('package/link', kind=tarfile.SYMTYPE, linkname='/outside'),
                   member('package/link', kind=tarfile.LNKTYPE, linkname='package/' + module.NATIVE),
                   member('package/' + module.NATIVE, b'duplicate'),
                   member('package/' + module.NATIVE.upper(), b'collision'),
                   member('package/postinstall.js', b'no')]
        for entry in entries:
            with self.subTest(path=entry[0].name, kind=entry[0].type):
                selection, source, routes = fixture(archive(extra=[entry]))
                with self.assertRaises(ValueError):
                    module.validated_binary(selection['registryBuildInputs'][module.KEY], lambda url, limit: routes[url])
        raw = gzip.decompress(archive()) + b'evil trailing archive'
        selection, source, routes = fixture(gzip.compress(raw))
        with self.assertRaises(ValueError):
            module.validated_binary(selection['registryBuildInputs'][module.KEY], lambda url, limit: routes[url])

    def test_manifest_contract_and_native_platform_reject(self):
        cases = [archive(manifest(**changes)) for changes in [
            {'scripts': {'postinstall': 'anything'}}, {'main': 'index.js'}, {'version': '2.0.1'},
            {'os': ['darwin']}, {'cpu': ['arm64']}, {'libc': ['musl']}]]
        cases += [archive(binary=b'not a library'), archive(binary=elf()[:18] + b'\xb7\0' + elf()[20:]),
                  archive(raw=b'{"name":"bad","name":"@farmfe/core-linux-x64-gnu","version":"2.0.0"}')]
        for body in cases:
            selection, source, routes = fixture(body)
            with self.assertRaises(ValueError):
                module.validated_binary(selection['registryBuildInputs'][module.KEY], lambda url, limit: routes[url])

    def test_existing_or_symlink_destinations_are_never_overwritten(self):
        self.destination.write_bytes(b'original')
        with self.assertRaises(FileExistsError):
            self.materialize()
        self.assertEqual(self.destination.read_bytes(), b'original')
        self.read.assert_not_called()
        self.destination.unlink()
        self.destination.symlink_to(self.root / 'outside')
        with self.assertRaises(FileExistsError):
            self.materialize()
        self.assertTrue(self.destination.is_symlink())
        self.assertFalse((self.root / 'outside').exists())

    def test_parent_symlink_and_output_collision_reject_before_network(self):
        binding = self.destination.parent
        binding.rmdir()
        outside = self.root / 'outside'
        outside.mkdir()
        binding.symlink_to(outside, target_is_directory=True)
        with self.assertRaises(OSError):
            self.materialize()
        self.read.assert_not_called()
        binding.unlink()
        binding.mkdir()
        self.output.parent.mkdir()
        self.output.write_text('original')
        with self.assertRaises(FileExistsError):
            self.materialize()
        self.assertEqual(self.output.read_text(), 'original')
        self.assertFalse(self.destination.exists())

    def test_other_destination_or_output_path_rejects(self):
        for destination, output in [(self.root / 'outside', self.output),
                                    (self.destination, self.root / 'different.json')]:
            with self.assertRaisesRegex(ValueError, 'fixed paths'):
                module.materialize(self.selection, self.root, destination, output, '/trusted/node',
                                   source_reader=lambda root: self.source, read=self.read, smoke=self.smoke)
        self.read.assert_not_called()

    def test_probe_receipt_requires_pinned_node_api_compile_and_execution(self):
        for changes in [{'node': 'v24.22.1'}, {'napi': ''}, {'platform': 'darwin'}, {'libc': 'musl'},
                        {'requiredExports': ['Compiler']}, {'resolver': False}, {'compile': False},
                        {'tsTransformed': False}, {'outputExecuted': False}, {'outputSha256': 'bad'},
                        {'outputBytes': 0}]:
            with self.subTest(changes=changes):
                self.smoke.return_value = proof(**changes)
                with self.assertRaisesRegex(ValueError, 'smoke contract'):
                    self.materialize()
                self.assertFalse(self.destination.exists())
                self.assertFalse(self.output.exists())

    def test_sanitized_environment_cannot_inherit_credentials_or_loader_overrides(self):
        secrets = {key: 'must-not-escape' for key in ['NPM_TOKEN', 'NODE_AUTH_TOKEN', 'GITHUB_TOKEN',
                   'ACTIONS_ID_TOKEN_REQUEST_TOKEN', 'NODE_OPTIONS', 'LD_PRELOAD', 'NODE_PATH',
                   'SSH_AUTH_SOCK', 'npm_config_registry']}
        with mock.patch.dict(os.environ, secrets):
            result = module.clean_environment('/trusted/bin/node', self.root)
        self.assertFalse(set(secrets) & set(result))
        self.assertEqual(result['HOME'], str(self.root))
        self.assertEqual(result['npm_config_ignore_scripts'], 'true')
        self.assertEqual(result['npm_config_userconfig'], '/dev/null')

    def test_production_smoke_has_no_node_version_override(self):
        node = shutil.which('node')
        if node is None:
            self.skipTest('Node unavailable')
        actual = subprocess.check_output([node, '--version']).strip().decode()
        if actual == module.NODE_VERSION:
            self.skipTest('The production Node version is installed')
        with self.assertRaisesRegex(ValueError, 'requires Node v22.20.0'):
            module.node_smoke(node, self.root / 'absent.node', self.source, self.root)
        self.assertFalse((self.root / 'runtime').exists())

    def test_real_source_reader_uses_committed_runtime_and_manifest(self):
        root = Path(__file__).resolve().parent.parent
        source = module.source_identity(root)
        self.assertEqual(source['manifest']['name'], module.NAME)
        self.assertRegex(source['sha'], '^[a-f0-9]{40}$')
        self.assertRegex(source['tree'], '^[a-f0-9]{40}$')
        self.assertIn('src/modules/module-helper.ts', source['runtime'])

    def test_public_fetch_refuses_redirect_status_or_changed_url(self):
        response = mock.MagicMock()
        response.__enter__.return_value = response
        response.getcode.return_value = 302
        response.geturl.return_value = 'https://registry.npmjs.org/a'
        opener = mock.Mock()
        opener.open.return_value = response
        with mock.patch.object(module.urllib.request, 'build_opener', return_value=opener):
            with self.assertRaises(ValueError):
                module.public_read('https://registry.npmjs.org/a', 100)
            response.getcode.return_value = 200
            response.geturl.return_value = 'https://evil.test/a'
            with self.assertRaises(ValueError):
                module.public_read('https://registry.npmjs.org/a', 100)
        self.assertIsNone(module.NoRedirect().redirect_request(None, None, 302, None, None, 'https://evil.test'))


if __name__ == '__main__':
    unittest.main()
