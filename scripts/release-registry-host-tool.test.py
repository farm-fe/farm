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


class PluginContracts(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix='farm-host-plugin-test-')
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.configure('replace-dirname')

    def configure(self, slug):
        self.slug = slug
        self.name = f'@farmfe/plugin-{slug}-linux-x64-gnu'
        self.key = 'npm:' + self.name
        self.manifest_path = f'rust-plugins/{slug}/npm/linux-x64-gnu/package.json'
        self.destination = self.root / f'rust-plugins/{slug}/npm/linux-x64-gnu/index.farm'
        self.output = self.root / f'release-work/registry-host-plugin-{slug}-material.json'
        value = manifest(name=self.name, main='index.farm')
        # The plugin archive has index.farm, never the Node core filename.
        stream = io.BytesIO()
        with tarfile.open(fileobj=stream, mode='w', format=tarfile.PAX_FORMAT) as tar:
            for item, data in [member('package/package.json', json.dumps(value).encode()),
                               member('package/index.farm', elf()), member('package/LICENSE', b'MIT')]:
                tar.addfile(item, io.BytesIO(data))
        self.body = gzip.compress(stream.getvalue(), mtime=0)
        self.selection, self.source, unused = fixture()
        row = {**self.selection['registryRetained'][module.KEY], 'name': self.name,
               'owner': f'@farmfe/plugin-{slug}', 'manifestPath': self.manifest_path,
               'tarballURL': f'https://registry.npmjs.org/@farmfe/plugin-{slug}-linux-x64-gnu/-/plugin-{slug}-linux-x64-gnu-2.0.0.tgz',
               'integrity': 'sha512-' + base64.b64encode(hashlib.sha512(self.body).digest()).decode(),
               'archiveSha256': hashlib.sha256(self.body).hexdigest(), 'archiveBytes': len(self.body)}
        self.selection['registryRetained'] = {self.key: row}
        self.selection['registryBuildInputs'] = {self.key: {**row, 'origin': 'registry-present'}}
        self.selection['nativeBuildCells'] = [{'package': module.KEY, 'owner': '@farmfe/core',
                                               'platform': 'linux-x64-gnu'}]
        self.selection['registrySnapshot'] = {'packages': {module.KEY: {'status': 'missing'}}}
        reseal(self.selection)
        self.source['manifests'] = {self.name: value}
        self.source['wrappers'] = {self.name: {'name': f'@farmfe/plugin-{slug}',
                                    'optionalDependencies': {self.name: '2.0.0'}}}
        self.destination.parent.mkdir(parents=True, exist_ok=True)
        (self.destination.parent / 'package.json').write_text(json.dumps(value))
        self.link = self.root / f'rust-plugins/{slug}/node_modules' / self.name
        self.link.parent.mkdir(parents=True, exist_ok=True)
        self.link.symlink_to(self.destination.parent, target_is_directory=True)
        core = self.root / module.DESTINATION
        core.parent.mkdir(parents=True, exist_ok=True)
        core.write_bytes(elf())
        metadata = {'name': self.name, 'version': '2.0.0',
                    'dist': {'tarball': row['tarballURL'], 'integrity': row['integrity']}}
        url = 'https://registry.npmjs.org/' + module.urllib.parse.quote(self.name, safe='') + '/2.0.0'
        self.routes = {url: json.dumps(metadata).encode(), row['tarballURL']: self.body}
        self.read = mock.Mock(side_effect=lambda url, limit: self.routes[url])
        self.smoke = mock.Mock(return_value=proof(plugin=self.name, rustPluginLoaded=True, pluginFixture=True))

    def materialize(self):
        return module.materialize(self.selection, self.root, self.destination, self.output,
                                  '/trusted/node', package=self.name, source_reader=lambda root: self.source,
                                  read=self.read, smoke=self.smoke)

    def test_each_allowlisted_plugin_materializes_fixed_workspace_file_not_node_modules(self):
        for slug in ['replace-dirname', 'worker']:
            if slug != self.slug:
                self.configure(slug)
            with self.subTest(plugin=slug):
                original = copy.deepcopy(self.selection)
                report = self.materialize()
                self.assertEqual(self.destination.read_bytes(), elf())
                self.assertTrue(self.link.is_symlink())
                self.assertEqual((self.link / 'index.farm').resolve(), self.destination)
                self.assertEqual(report['name'], self.name)
                self.assertEqual(report['owner'], f'@farmfe/plugin-{slug}')
                self.assertEqual(report['destination'], str(self.destination.relative_to(self.root)))
                self.assertEqual(json.loads(self.output.read_text()), report)
                self.assertFalse(report['sourceNativeArtifact'])
                self.assertFalse(report['publicationCandidate'])
                self.assertTrue(report['smoke']['rustPluginLoaded'])
                self.assertTrue(report['smoke']['pluginFixture'])
                self.assertEqual(report['compiler']['origin'], 'source-build')
                self.assertEqual(report['compiler']['binarySha256'], hashlib.sha256(elf()).hexdigest())
                self.assertEqual(original, self.selection)
                self.assertEqual(self.read.call_count, 2)

    def test_plugin_identity_source_version_and_rebuild_overlap_fail_before_transfer(self):
        baseline = copy.deepcopy(self.selection)
        for field, value in [('owner', '@farmfe/core'), ('name', module.NAME),
                             ('manifestPath', module.MANIFEST), ('platform', 'linux-x64-musl'),
                             ('version', '2.0.1')]:
            with self.subTest(field=field):
                self.selection = copy.deepcopy(baseline)
                for section in ['registryBuildInputs', 'registryRetained']:
                    self.selection[section][self.key][field] = value
                reseal(self.selection)
                with self.assertRaises(ValueError):
                    self.materialize()
                self.read.assert_not_called()
        for field in ['nativeBuildCells', 'preparationNativeCells']:
            self.selection = copy.deepcopy(baseline)
            self.selection.setdefault(field, []).append({'package': self.key})
            reseal(self.selection)
            with self.assertRaises(ValueError):
                self.materialize()
        self.read.assert_not_called()

    def test_unrecognized_plugin_rejected_before_read_or_write(self):
        self.name = '@farmfe/plugin-arbitrary-linux-x64-gnu'
        with self.assertRaisesRegex(ValueError, 'allowlist'):
            self.materialize()
        self.read.assert_not_called()

    def test_plugin_wrapper_link_must_resolve_to_workspace_platform(self):
        self.link.unlink()
        outside = self.root / 'another-package'
        outside.mkdir()
        self.link.symlink_to(outside, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, 'wrapper'):
            self.materialize()
        self.read.assert_not_called()
        self.assertFalse(self.destination.exists())
        self.assertFalse(any(outside.iterdir()))

    def test_plugin_wrapper_missing_or_changed_manifest_fails(self):
        self.link.unlink()
        with self.assertRaises((ValueError, FileNotFoundError)):
            self.materialize()
        self.link.symlink_to(self.destination.parent, target_is_directory=True)
        (self.destination.parent / 'package.json').write_text(json.dumps(manifest(name=self.name, main='evil.js')))
        with self.assertRaisesRegex(ValueError, 'manifest'):
            self.materialize()
        self.read.assert_not_called()

    def test_symlink_or_existing_plugin_native_and_parent_never_overwritten(self):
        self.destination.symlink_to(self.root / 'outside.farm')
        with self.assertRaises(FileExistsError):
            self.materialize()
        self.assertFalse((self.root / 'outside.farm').exists())
        self.destination.unlink()
        self.destination.write_bytes(b'untouched')
        with self.assertRaises(FileExistsError):
            self.materialize()
        self.assertEqual(self.destination.read_bytes(), b'untouched')
        self.read.assert_not_called()

    def test_plugin_requires_selected_verified_core_and_never_builds_fallback(self):
        self.selection['nativeBuildCells'] = []
        reseal(self.selection)
        with self.assertRaisesRegex(ValueError, 'core'):
            self.materialize()
        self.read.assert_not_called()
        self.smoke.assert_not_called()

    def test_source_core_cell_can_be_shared_by_publication_and_host_preparation(self):
        self.selection['preparationNativeCells'] = copy.deepcopy(self.selection['nativeBuildCells'])
        reseal(self.selection)
        report = self.materialize()
        self.assertEqual(report['compiler']['origin'], 'source-build')

    def test_conflicting_duplicate_core_cell_cannot_authorize_plugin_loading(self):
        self.selection['preparationNativeCells'] = [{'package': module.KEY, 'owner': '@farmfe/core',
                                                   'platform': 'linux-x64-musl'}]
        reseal(self.selection)
        with self.assertRaisesRegex(ValueError, 'core'):
            self.materialize()
        self.read.assert_not_called()

    def test_plugin_rejects_unverified_registry_core_even_when_binding_exists(self):
        core_selection, _, _ = fixture()
        for section in ['registryBuildInputs', 'registryRetained']:
            self.selection[section].update(core_selection[section])
        self.selection['nativeBuildCells'] = []
        reseal(self.selection)
        with self.assertRaises((ValueError, FileNotFoundError)):
            self.materialize()
        self.read.assert_not_called()

    def test_plugin_abi_or_fixture_failure_never_materializes_or_rebuilds(self):
        for changes in [{'rustPluginLoaded': False}, {'pluginFixture': False}, {'plugin': module.NAME}]:
            self.smoke.return_value = proof(plugin=self.name, rustPluginLoaded=True, pluginFixture=True) | changes
            with self.assertRaisesRegex(ValueError, 'smoke contract'):
                self.materialize()
            self.assertFalse(self.destination.exists())
            self.assertFalse(self.output.exists())
        self.smoke.side_effect = ValueError('native Farm ABI mismatch')
        with self.assertRaisesRegex(ValueError, 'ABI mismatch'):
            self.materialize()
        self.assertFalse(self.destination.exists())

    def test_plugin_integrity_error_never_runs_native_code(self):
        url = self.selection['registryBuildInputs'][self.key]['tarballURL']
        self.routes[url] += b'tamper'
        with self.assertRaises(ValueError):
            self.materialize()
        self.smoke.assert_not_called()
        self.assertFalse(self.destination.exists())

    def test_verified_registry_core_receipt_is_bound_to_source_plan_and_binary(self):
        core_selection, _, core_routes = fixture()
        for section in ['registryBuildInputs', 'registryRetained']:
            self.selection[section].update(core_selection[section])
        self.selection['nativeBuildCells'] = []
        reseal(self.selection)
        self.routes.update(core_routes)
        (self.root / module.DESTINATION).unlink()
        core_report = module.materialize(self.selection, self.root, self.root / module.DESTINATION,
                                        self.root / module.OUTPUT, '/trusted/node',
                                        source_reader=lambda root: self.source, read=self.read,
                                        smoke=mock.Mock(return_value=proof()))
        self.read.reset_mock()
        for changes in [{'sourceSHA': 'f' * 40}, {'planHash': 'f' * 64}, {'binarySha256': 'f' * 64},
                        {'registryBytesVerified': 1}, {'publicationCandidate': True},
                        {'smoke': proof(compile=False)}]:
            (self.root / module.OUTPUT).chmod(0o644)
            (self.root / module.OUTPUT).write_text(json.dumps(core_report | changes))
            with self.subTest(changes=changes), self.assertRaises(ValueError):
                self.materialize()
            self.assertFalse(self.destination.exists())
        self.read.assert_not_called()
        (self.root / module.OUTPUT).write_text(json.dumps(core_report))
        report = self.materialize()
        self.assertEqual(report['compiler']['origin'], 'registry-present')
        self.assertEqual(report['coreBinarySha256'], core_report['binarySha256'])

    def test_plugin_archive_boundaries_are_enforced_independently(self):
        value = manifest(name=self.name, main='index.farm')
        cases = [member('package/../outside', b'bad'),
                 member('package/index.farm', b'duplicate'),
                 member('package/INDEX.FARM', b'collision'),
                 member('package/postinstall.js', b'bad'),
                 member('package/extra', kind=tarfile.SYMTYPE, linkname='/outside'),
                 member('package/extra', kind=tarfile.LNKTYPE, linkname='package/index.farm')]
        for extra in cases:
            stream = io.BytesIO()
            with tarfile.open(fileobj=stream, mode='w', format=tarfile.PAX_FORMAT) as tar:
                for item, body in [member('package/package.json', json.dumps(value).encode()),
                                   member('package/index.farm', elf()), extra]:
                    tar.addfile(item, io.BytesIO(body))
            body = gzip.compress(stream.getvalue(), mtime=0)
            self.replace_archive(body)
            with self.subTest(name=extra[0].name), self.assertRaises(ValueError):
                self.materialize()
            self.smoke.assert_not_called()
            self.assertFalse(self.destination.exists())

    def replace_archive(self, body):
        for section in ['registryBuildInputs', 'registryRetained']:
            self.selection[section][self.key].update(
                integrity='sha512-' + base64.b64encode(hashlib.sha512(body).digest()).decode(),
                archiveSha256=hashlib.sha256(body).hexdigest(), archiveBytes=len(body))
        reseal(self.selection)
        row = self.selection['registryBuildInputs'][self.key]
        self.routes[row['tarballURL']] = body
        metadata_url = 'https://registry.npmjs.org/' + module.urllib.parse.quote(self.name, safe='') + '/2.0.0'
        self.routes[metadata_url] = json.dumps({'name': self.name, 'version': '2.0.0',
            'dist': {'tarball': row['tarballURL'], 'integrity': row['integrity']}}).encode()

    def test_plugin_archive_rejects_wrong_platform_lifecycle_script_and_non_elf(self):
        for changes, binary in [({'os': ['darwin']}, elf()), ({'libc': ['musl']}, elf()),
                                ({'cpu': ['arm64']}, elf()), ({'name': module.NAME}, elf()),
                                ({'version': '2.0.1'}, elf()), ({'scripts': {'install': 'evil'}}, elf()),
                                ({'main': module.NATIVE}, elf()), ({}, b'not ELF'),
                                ({}, elf()[:18] + b'\xb7\0' + elf()[20:])]:
            value = manifest(name=self.name, main='index.farm') | changes
            stream = io.BytesIO()
            with tarfile.open(fileobj=stream, mode='w', format=tarfile.PAX_FORMAT) as tar:
                for item, body in [member('package/package.json', json.dumps(value).encode()),
                                   member('package/index.farm', binary)]:
                    tar.addfile(item, io.BytesIO(body))
            self.replace_archive(gzip.compress(stream.getvalue(), mtime=0))
            with self.subTest(changes=changes), self.assertRaises(ValueError):
                self.materialize()
            self.smoke.assert_not_called()
            self.assertFalse(self.destination.exists())

    def test_symlinked_core_or_manifest_or_plugin_parent_never_read_or_written(self):
        core = self.root / module.DESTINATION
        core.rename(core.with_suffix('.original'))
        core.symlink_to(core.with_suffix('.original'))
        with self.assertRaises(OSError):
            self.materialize()
        core.unlink()
        core.with_suffix('.original').rename(core)
        manifest_path = self.root / self.manifest_path
        manifest_path.rename(manifest_path.with_suffix('.original'))
        manifest_path.symlink_to(manifest_path.with_suffix('.original'))
        with self.assertRaises(OSError):
            self.materialize()
        manifest_path.unlink()
        manifest_path.with_suffix('.original').rename(manifest_path)
        platform = self.destination.parent
        platform.rename(platform.with_name('original'))
        platform.symlink_to(platform.with_name('original'), target_is_directory=True)
        with self.assertRaises(OSError):
            self.materialize()
        self.read.assert_not_called()
        self.assertFalse((platform.with_name('original') / 'index.farm').exists())

    def test_plugin_only_uses_metadata_version_selection_and_measures_transferred_bytes(self):
        for section in ['registryBuildInputs', 'registryRetained']:
            self.selection[section][self.key].pop('archiveSha256')
            self.selection[section][self.key].pop('archiveBytes')
        reseal(self.selection)
        report = self.materialize()
        self.assertEqual(report['archiveSha256'], hashlib.sha256(self.body).hexdigest())
        self.assertEqual(report['archiveBytes'], len(self.body))
        self.assertNotIn('archiveSha256', self.selection['registryBuildInputs'][self.key])

    def test_core_changed_during_smoke_fails_before_plugin_output(self):
        def smoke(*args, **kwargs):
            (self.root / module.DESTINATION).write_bytes(elf() + b'changed')
            return proof(plugin=self.name, rustPluginLoaded=True, pluginFixture=True)
        self.smoke.side_effect = smoke
        with self.assertRaisesRegex(ValueError, 'core changed'):
            self.materialize()
        self.assertFalse(self.destination.exists())
        self.assertFalse(self.output.exists())

    def test_production_plugin_smoke_has_real_loader_and_behavioral_assertions(self):
        script = Path(__file__).with_name('release-registry-host-smoke.mjs').read_text()
        self.assertIn("rustPlugins: plugin ? [[plugin, '{}']] : []", script)
        self.assertIn('compiler.compileSync()', script)
        self.assertIn('directory: __dirname, file: __filename, url: import.meta.url', script)
        self.assertIn('assert.deepEqual(result.location', script)
        self.assertIn('registry-fixture-worker.ts?worker&inline', script)
        self.assertIn('Inline worker must contain a real compiler resource', script)

    def test_source_reader_records_committed_plugin_manifests_and_wrappers(self):
        source = module.source_identity(Path(__file__).resolve().parent.parent)
        for slug in ['replace-dirname', 'worker']:
            name = f'@farmfe/plugin-{slug}-linux-x64-gnu'
            self.assertEqual(source['manifests'][name]['name'], name)
            self.assertEqual(source['manifests'][name]['main'], 'index.farm')
            self.assertEqual(source['wrappers'][name]['optionalDependencies'][name], source['manifests'][name]['version'])


if __name__ == '__main__':
    unittest.main()
