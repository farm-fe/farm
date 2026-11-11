"""Original-byte npm manifest security contracts; no network or npm writes."""
# cspell:words importlib subtests gzip PAX SCHILY npmjs
import copy
import gzip
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tarfile
import shutil
import subprocess
import tempfile
import unittest
from unittest import mock

spec = importlib.util.spec_from_file_location('release_npm_tarball', Path(__file__).with_name('release-npm-tarball.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def manifest(**extra):
    return {'name': '@farmfe/core', 'version': '2.0.0',
            'repository': {'type': 'git', 'url': 'https://github.com/farm-fe/farm'}, **extra}


def row(body, **extra):
    return {'kind': 'npm', 'name': '@farmfe/core', 'version': '2.0.0',
            'archiveSha256': hashlib.sha256(body).hexdigest(), **extra}


def member(name, body=b'', kind=tarfile.REGTYPE, **extra):
    info = tarfile.TarInfo(name)
    info.type = kind
    info.size = len(body)
    for key, value in extra.items():
        setattr(info, key, value)
    return info, body


def archive(value=None, *, members=None, raw=None):
    if members is None:
        if raw is None:
            raw = json.dumps(manifest() if value is None else value).encode()
        members = [member('package/package.json', raw), member('package/index.js', b'original frozen code')]
    output = io.BytesIO()
    with tarfile.open(fileobj=output, mode='w', format=tarfile.PAX_FORMAT) as tar:
        for info, body in members:
            tar.addfile(info, io.BytesIO(body))
    return gzip.compress(output.getvalue(), mtime=0)


class Contracts(unittest.TestCase):
    def reject(self, body, pattern=None, **changes):
        with self.assertRaisesRegex(ValueError, pattern or '.'):
            module.validate(row(body, **changes), body)

    def test_valid_original_bytes_and_record_are_not_modified(self):
        value = manifest(scripts={'prepublishOnly': 'exit 100'}, files=['dist'],
                         repository={'type': 'git', 'url': 'git+https://github.com/farm-fe/farm.git', 'directory': 'packages/core'})
        body = archive(value)
        record = row(body)
        previous = copy.deepcopy(record)
        digest = hashlib.sha256(body).hexdigest()
        self.assertEqual(module.validate(record, body), value)
        self.assertEqual(record, previous)
        self.assertEqual(hashlib.sha256(body).hexdigest(), digest)

    def test_canonical_repository_urls_and_string_normalization_are_allowed(self):
        for url in sorted(module.REPOSITORIES):
            for value in [url, {'url': url}, {'type': 'git', 'url': url, 'directory': 'packages/core/'}]:
                with self.subTest(repository=value):
                    body = archive(manifest(repository=value))
                    self.assertEqual(module.validate(row(body), body)['repository'], value)

    def test_invalid_or_ambiguous_repository_metadata_is_rejected(self):
        for value in [None, {}, [], True, {'url': None}, {'url': []}, {'type': 'svn', 'url': next(iter(module.REPOSITORIES))},
                      'https://github.com/other/farm', 'https://github.com/farm-fe/farm-other',
                      'https://github.com.evil.invalid/farm-fe/farm', 'https://github.com@evil.invalid/farm-fe/farm',
                      'https://evil.invalid@github.com/farm-fe/farm', 'https://github.com/farm-fe/farm?x=1',
                      'https://github.com/farm-fe/farm#main', 'https://github.com/farm-fe/farm/tree/main/packages/create-farm',
                      'https://github.com/farm-fe/farm/', 'https://github.com/FARM-FE/farm',
                      'https://github.com/farm-fe/%66arm', 'https://github.com:443/farm-fe/farm',
                      'https://github.com/farm-fe/farm\n', 'http://github.com/farm-fe/farm']:
            with self.subTest(repository=value):
                self.reject(archive(manifest(repository=value)), 'repository')
        for repositories in [[], [next(iter(module.REPOSITORIES))], ['https://evil.invalid/repo'], None]:
            self.reject(archive(manifest(repositories=repositories)), 'Legacy')

    def test_safe_publish_config_is_explicitly_allowed(self):
        for registry in [module.REGISTRY, module.REGISTRY[:-1]]:
            value = manifest(publishConfig={'registry': registry, 'access': 'public',
                                           'provenance': True, 'tag': 'farm-release-2.0.0'})
            body = archive(value)
            self.assertEqual(module.validate(row(body), body), value)
        body = archive(manifest(publishConfig={}))
        self.assertEqual(module.validate(row(body), body)['publishConfig'], {})

    def test_unknown_config_and_credentials_cannot_override_cli(self):
        for key in ['@farmfe:registry', '//registry.npmjs.org/:_authToken', '_authToken', '_auth',
                    'username', '_password', 'email', 'otp', 'always-auth', 'provenance-file',
                    'userconfig', 'globalconfig', 'ignore-scripts', 'dry-run', 'force', 'scope',
                    'proxy', 'https-proxy', 'certfile', 'keyfile', 'cafile', 'strict-ssl',
                    'registry ', 'Registry', '__proto__', 'constructor', 'future-new-config']:
            with self.subTest(key=key):
                self.reject(archive(manifest(publishConfig={key: 'SENTINEL-SECRET'})), 'Unsupported')

    def test_invalid_safe_config_values_are_rejected(self):
        for key, values in {
            'registry': ['https://evil.invalid/', 'http://registry.npmjs.org/', 'https://registry.npmjs.org/x', None, [], {}],
            'access': ['restricted', '', True, None, [], {}],
            'provenance': [False, 'true', 'false', 1, 0, None, [], {}],
            'tag': ['latest', 'farm-release-2.0.1', '', None, [], {}],
        }.items():
            for value in values:
                with self.subTest(key=key, value=value):
                    self.reject(archive(manifest(publishConfig={key: value})), 'publishConfig')
        for value in [None, False, '', [], ['registry']]:
            self.reject(archive(manifest(publishConfig=value)), 'publishConfig')

    def test_errors_never_echo_attacker_controlled_secrets(self):
        for value in [manifest(repository='SENTINEL-SECRET'), manifest(publishConfig={'SENTINEL-SECRET': 'SENTINEL-SECRET'})]:
            body = archive(value)
            with self.assertRaises(ValueError) as error:
                module.validate(row(body), body)
            self.assertNotIn('SENTINEL-SECRET', str(error.exception))

    def test_top_level_tag_cannot_override_controlled_publication(self):
        for value in ['latest', 'unexpected-channel', 'farm-release-2.0.0']:
            with self.subTest(value=value):
                self.reject(archive(manifest(tag=value)), 'Top-level npm tag')

    def test_identity_and_private_flags_are_checked(self):
        for changes in [{'name': 'other'}, {'version': '2.0.1'}, {'name': None}, {'version': 2}]:
            self.reject(archive(manifest(**changes)), 'identity')
        for value in [True, 'false', 0, None, [], {}]:
            self.reject(archive(manifest(private=value)), 'Private')
        body = archive(manifest(private=False))
        self.assertFalse(module.validate(row(body), body)['private'])

    def test_invalid_record_and_checksum_are_rejected_before_archive_parse(self):
        body = archive()
        for changes in [{'kind': 'cargo'}, {'name': 'a/../b'}, {'name': '@scope/'}, {'name': '.hidden'},
                        {'name': 'A'}, {'name': 'x' * 215}, {'version': 'v2.0.0'}, {'version': '02.0.0'},
                        {'version': '2.0.0-beta.1'}, {'version': '2.0.0+build'}, {'archiveSha256': '0' * 64}]:
            with self.subTest(changes=changes):
                self.reject(body, **changes)
        with self.assertRaises(ValueError):
            module.validate(None, body)
        with self.assertRaises(ValueError):
            module.validate(row(body), bytearray(body))

    def test_duplicate_json_keys_and_nonstandard_json_are_rejected(self):
        for raw in [b'{"name":"bad","name":"@farmfe/core","version":"2.0.0"}',
                    b'{"name":"@farmfe/core","version":"2.0.0","publishConfig":{"provenance":false,"provenance":true}}',
                    b'{"x":NaN}', b'{"x":Infinity}', b'{"x":-Infinity}', b'{', b'\xff', b'\xef\xbb\xbf{}',
                    b'[' * 1100 + b']' * 1100, b'null', b'[]', b'1']:
            self.reject(archive(raw=raw))

    def test_requires_exact_manifest_and_no_duplicate_entries(self):
        data = json.dumps(manifest()).encode()
        for entries in [[], [member('package/index.js', b'x')],
                        [member('package/package.json', data), member('package/package.json', data)],
                        [member('package/Package.json', data)],
                        [member('package/package.json', data), member('package/PACKAGE.JSON', data)],
                        [member('package/package.json/', kind=tarfile.DIRTYPE)]]:
            self.reject(archive(members=entries))

    def test_paths_are_canonical_and_within_package(self):
        data = json.dumps(manifest()).encode()
        for path in ['/package/package.json', './package/package.json', 'other/package.json',
                     'package/../package.json', 'package/./package.json', 'package//package.json',
                     'package\\package.json', 'package/package.json ', 'package/package.json.',
                     'package/AUX', 'package/COM1.txt', 'package/c:/package.json', 'package/x\n.json',
                     'package/e\u0301.json']:
            with self.subTest(path=path):
                self.reject(archive(members=[member('package/package.json', data), member(path, b'{}')]))

    def test_special_members_and_file_directory_conflicts_are_rejected(self):
        data = json.dumps(manifest()).encode()
        base = [member('package/package.json', data)]
        for kind in [tarfile.SYMTYPE, tarfile.LNKTYPE, tarfile.CHRTYPE, tarfile.BLKTYPE, tarfile.FIFOTYPE, tarfile.GNUTYPE_SPARSE]:
            with self.subTest(kind=kind):
                self.reject(archive(members=base + [member('package/link', kind=kind, linkname='package/package.json')]))
        for entries in [[member('package/a', b'x'), member('package/a/b', b'y')],
                        [member('package/a/b', b'x'), member('package/a', b'y')],
                        [member('package/a', b'x'), member('package/a/', kind=tarfile.DIRTYPE)],
                        [member('package/a/', b'payload', kind=tarfile.DIRTYPE)],
                        [member('package/a', b'x', linkname='package/package.json')]]:
            self.reject(archive(members=base + entries))

    def test_normal_directories_and_ustar_long_prefix_are_supported(self):
        data = json.dumps(manifest()).encode()
        entries = [member('package/', kind=tarfile.DIRTYPE), member('package/package.json', data),
                   member('package/dist/', kind=tarfile.DIRTYPE), member('package/dist/index.js', b'x')]
        body = archive(members=entries)
        self.assertEqual(module.validate(row(body), body)['name'], '@farmfe/core')
        long = 'package/' + 'a' * 80 + '/' + 'b' * 80 + '.js'
        output = io.BytesIO()
        with tarfile.open(fileobj=output, mode='w', format=tarfile.USTAR_FORMAT) as tar:
            for info, payload in entries + [member(long, b'x')]:
                tar.addfile(info, io.BytesIO(payload))
        body = gzip.compress(output.getvalue())
        module.validate(row(body), body)

    def test_unambiguous_pax_long_paths_and_standard_metadata_are_supported(self):
        entries = [member('package/package.json', json.dumps(manifest()).encode()),
                   member('package/' + 'x' * 140 + '.js', b'code', pax_headers={'mtime': '1.5'})]
        body = archive(members=entries)
        module.validate(row(body), body)
        body = archive(members=[member('package/package.json', json.dumps(manifest()).encode(),
                                      pax_headers={'size': str(len(json.dumps(manifest()).encode())),
                                                   'linkpath': '', 'SCHILY.ino': '12'})])
        module.validate(row(body), body)

    def test_pax_cannot_hide_paths_sizes_or_sparse_overrides(self):
        data = json.dumps(manifest()).encode()
        for pax in [{'path': '../package/package.json'}, {'path': 'package/./package.json'},
                    {'size': str(len(data) + 1)}, {'GNU.sparse.map': '0,1'}, {'linkpath': 'outside'},
                    {'path': 'package/package.json\n'}, {'unknown': 'x'}]:
            self.reject(archive(members=[member('package/package.json', data, pax_headers=pax)]))
        body = archive(members=[member('package/package.json', data),
                                member('package/other', data, pax_headers={'path': 'package/package.json'})])
        self.reject(body, 'Duplicate')

    def test_global_and_stacked_or_orphan_extensions_are_rejected(self):
        data = json.dumps(manifest()).encode()
        pax = b'22 path=package/other\n'
        for kind in [tarfile.XGLTYPE, tarfile.GNUTYPE_LONGNAME, tarfile.GNUTYPE_LONGLINK]:
            self.reject(archive(members=[member('PaxHeader', pax, kind), member('package/package.json', data)]))
        self.reject(archive(members=[member('PaxHeader', b'15 mtime=1.2345\n', tarfile.XHDTYPE)]))
        self.reject(archive(members=[member('PaxHeader', b'15 mtime=1.2345\n', tarfile.XHDTYPE),
                                    member('PaxHeader', b'15 mtime=1.2345\n', tarfile.XHDTYPE)]))

    def test_pax_path_overrides_ustar_prefix_as_in_npm_read_entry(self):
        data = json.dumps(manifest()).encode()
        body = archive(members=[member('package/package.json', data, pax_headers={'path': 'package/package.json'})])
        raw = bytearray(gzip.decompress(body))
        # The first PAX header and padded data consume 1024 bytes.
        offset = 1024
        raw[offset + 345:offset + 352] = b'package'
        raw[offset + 148:offset + 156] = b'        '
        checksum = sum(raw[offset:offset + 512])
        raw[offset + 148:offset + 156] = ('%06o\0 ' % checksum).encode()
        body = gzip.compress(raw)
        self.assertEqual(module.validate(row(body), body)['name'], '@farmfe/core')

    def test_semver_safe_integer_boundary_and_unscoped_names(self):
        version = '9007199254740991.0.0'
        body = archive(manifest(version=version))
        module.validate(row(body, version=version), body)
        version = '9007199254740992.0.0'
        body = archive(manifest(version=version))
        self.reject(body, 'SemVer integer', version=version)
        for name in ['create-farm', 'create-farm-plugin', '@farmfe/core-linux-x64-gnu']:
            body = archive(manifest(name=name))
            module.validate(row(body, name=name), body)
        for name in ['node_modules', 'favicon.ico']:
            body = archive(manifest(name=name))
            self.reject(body, 'name', name=name)

    def test_real_offline_npm_pack_ordinary_ustar_and_pax_archives(self):
        # Real CLI packing, without auth, registry reads, lifecycle scripts or
        # installed dependencies. This also catches parser/producer drift.
        node = shutil.which('node')
        if node is None:
            self.skipTest('Node installation unavailable for offline pack contract')
        node = Path(node).resolve()
        cli = node.parent.parent / 'lib/node_modules/npm/bin/npm-cli.js'
        if not cli.is_file():
            self.skipTest('Bundled npm unavailable for offline pack contract')
        with tempfile.TemporaryDirectory(prefix='farm-offline-pack-') as temp:
            root = Path(temp)
            package, output = root / 'package', root / 'output'
            package.mkdir()
            output.mkdir()
            user, global_config = root / 'user.npmrc', root / 'global.npmrc'
            user.write_text('')
            global_config.write_text('')
            environment = {'PATH': str(node.parent), 'HOME': str(root),
                           'NPM_CONFIG_OFFLINE': 'true', 'NPM_CONFIG_USERCONFIG': str(user),
                           'NPM_CONFIG_GLOBALCONFIG': str(global_config),
                           'NPM_CONFIG_CACHE': str(root / 'cache'),
                           'NPM_CONFIG_AUDIT': 'false', 'NPM_CONFIG_FUND': 'false'}
            value = manifest(scripts={key: 'exit 99' for key in ['prepack', 'prepare', 'postpack']})
            (package / 'package.json').write_text(json.dumps(value))
            (package / 'index.js').write_text('export default 42;')
            for kind in ['ordinary', 'ustar', 'pax']:
                with self.subTest(format=kind):
                    if kind == 'ustar':
                        directory = package / ('a' * 80)
                        directory.mkdir()
                        (directory / ('b' * 80 + '.js')).write_text('export default 1;')
                    if kind == 'pax':
                        (package / ('x' * 140 + '.js')).write_text('export default 2;')
                    result = subprocess.run([str(node), str(cli), 'pack', '--offline', '--ignore-scripts',
                                             '--json', '--pack-destination', str(output)], cwd=package,
                                            env=environment, capture_output=True, text=True, timeout=60)
                    self.assertEqual(result.returncode, 0, result.stderr)
                    body = (output / json.loads(result.stdout)[0]['filename']).read_bytes()
                    self.assertEqual(module.validate(row(body), body)['name'], value['name'])
                    with tarfile.open(fileobj=io.BytesIO(body), mode='r:gz') as tar:
                        members = tar.getmembers()
                    if kind == 'ustar':
                        self.assertTrue(any(len(item.name) > 100 and not item.pax_headers for item in members))
                    if kind == 'pax':
                        self.assertTrue(any('path' in item.pax_headers for item in members))

    def test_duplicate_pax_fields_are_rejected(self):
        line = b'11 mtime=1\n'
        self.reject(archive(members=[member('PaxHeader', line + line, tarfile.XHDTYPE)]), 'Duplicate')

    def test_compressed_manifest_member_and_expansion_limits(self):
        body = archive()
        with mock.patch.object(module, 'MAX_COMPRESSED', len(body)):
            module.validate(row(body), body)
        with mock.patch.object(module, 'MAX_COMPRESSED', len(body) - 1):
            self.reject(body, 'Compressed')
        data = json.dumps(manifest()).encode()
        body = archive(raw=data)
        with mock.patch.object(module, 'MAX_MANIFEST', len(data)):
            module.validate(row(body), body)
        with mock.patch.object(module, 'MAX_MANIFEST', len(data) - 1):
            self.reject(body, 'manifest exceeds')
        expanded = len(gzip.decompress(body))
        with mock.patch.object(module, 'MAX_EXPANDED', expanded):
            module.validate(row(body), body)
        with mock.patch.object(module, 'MAX_EXPANDED', expanded - 1):
            self.reject(body, 'Expanded')
        with mock.patch.object(module, 'MAX_MEMBERS', 2):
            module.validate(row(body), body)
        with mock.patch.object(module, 'MAX_MEMBERS', 1):
            self.reject(body, 'member count')
        with mock.patch.object(module, 'MAX_EXPANDED', 2048):
            self.reject(archive(members=[member('package/huge', b'x' * 4096)]), 'Expanded')
        with mock.patch.object(module, 'MAX_PAX', 4):
            self.reject(archive(members=[member('package/' + 'x' * 140, b'x')]), 'PAX header')

    def test_gzip_corruption_truncation_padding_and_trailing_archives(self):
        body = archive()
        raw = gzip.decompress(body)
        mutated = bytearray(raw)
        mutated[0] ^= 1
        corrupt_crc = bytearray(body)
        corrupt_crc[-8] ^= 1
        for bad in [b'', b'not gzip', body[:-2], bytes(corrupt_crc), gzip.compress(raw[:-1]),
                    gzip.compress(raw[:513]), gzip.compress(bytes(mutated)),
                    gzip.compress(raw + b'x' * 512), body + body]:
            with self.subTest(size=len(bad)):
                self.reject(bad)
        padding = bytearray(raw)
        size = len(json.dumps(manifest()).encode())
        padding[512 + size] = 1
        self.reject(gzip.compress(padding), 'padding')
        # A valid gzip trailer with nonzero data after tar EOF is still invalid.
        self.reject(body + gzip.compress(b'junk'), 'trailing')


if __name__ == '__main__':
    unittest.main()
