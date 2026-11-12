"""Restore allowlisted exact registry GNU host tools, never source artifacts."""
# cspell:words importlib urllib npmjs userconfig globalconfig glibc NOSYSTEM RDONLY WRONLY CREAT fstat NONBLOCK
import argparse
import base64
from contextlib import contextmanager
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import tarfile
import tempfile
import time
import urllib.parse
import urllib.request


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(file))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


archive = load('registry_host_archive', 'release-npm-tarball.py')
NAME = '@farmfe/core-linux-x64-gnu'
KEY = 'npm:' + NAME
PLATFORM = 'linux-x64-gnu'
NATIVE = 'farm.linux-x64-gnu.node'
MANIFEST = 'packages/core/npm/linux-x64-gnu/package.json'
DESTINATION = 'packages/core/binding/' + NATIVE
OUTPUT = 'release-work/registry-host-material.json'
NODE_VERSION = 'v22.20.0'
EXPORTS = ['Compiler', 'DocsSnapshot', 'OutputDirectoryLock', 'Resolver']
MAX_JSON = 2 * 1024 ** 2
HOST_TOOLS = {NAME: {'owner': '@farmfe/core', 'manifest': MANIFEST, 'native': NATIVE,
                     'destination': DESTINATION, 'output': OUTPUT}}
for _slug in ['replace-dirname', 'worker']:
    HOST_TOOLS[f'@farmfe/plugin-{_slug}-linux-x64-gnu'] = {
        'owner': f'@farmfe/plugin-{_slug}',
        'manifest': f'rust-plugins/{_slug}/npm/linux-x64-gnu/package.json',
        'wrapper': f'rust-plugins/{_slug}/package.json',
        'link': f'rust-plugins/{_slug}/node_modules/@farmfe/plugin-{_slug}-linux-x64-gnu',
        'native': 'index.farm', 'destination': f'rust-plugins/{_slug}/npm/linux-x64-gnu/index.farm',
        'output': f'release-work/registry-host-plugin-{_slug}-material.json'}


def specification(package):
    check(isinstance(package, str) and package in HOST_TOOLS, 'Registry host package not in allowlist')
    return HOST_TOOLS[package]


def check(value, message):
    if not value:
        raise ValueError(message)


def parse_json(body):
    return json.loads(body, object_pairs_hook=archive.unique_object,
                      parse_constant=archive.invalid_constant)


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':'),
                                    ensure_ascii=True, allow_nan=False).encode()).hexdigest()


def safe_url(url):
    check(isinstance(url, str) and len(url) <= 2048, 'Registry URL invalid')
    parsed = urllib.parse.urlsplit(url)
    check(parsed.scheme == 'https' and parsed.hostname == 'registry.npmjs.org'
          and parsed.port in [None, 443] and not parsed.username and not parsed.password
          and not parsed.fragment and not parsed.query and parsed.path.startswith('/')
          and not any(ord(char) < 33 or ord(char) == 127 for char in url),
          'Registry URL origin or fields differ')
    return url


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, file, code, message, headers, new_url):
        return None


def public_read(url, limit):
    """Public GET only, no authentication, redirects, or unbounded body reads."""
    safe_url(url)
    request = urllib.request.Request(url, headers={
        'Accept': 'application/json', 'User-Agent': 'Farm-registry-host-tool'}, method='GET')
    deadline = time.monotonic() + 90
    chunks, count = [], 0
    with urllib.request.build_opener(NoRedirect()).open(request, timeout=20) as response:
        check(response.getcode() == 200 and response.geturl() == url,
              'Registry response status or destination differs')
        while True:
            check(time.monotonic() < deadline, 'Registry host transfer deadline exceeded')
            chunk = response.read(min(1024 ** 2, limit - count + 1))
            count += len(chunk)
            check(count <= limit, 'Registry host transfer exceeds byte bound')
            if not chunk:
                break
            chunks.append(chunk)
    check(count > 0, 'Registry host response empty')
    return b''.join(chunks)


def clean_environment(node, scratch):
    # Do not inherit token variables, NODE_OPTIONS, npm configuration, loader
    # paths, agent sockets, HOME credentials, or OIDC request credentials.
    return {'PATH': str(Path(node).parent) + ':/usr/bin:/bin', 'HOME': str(scratch),
            'TMPDIR': str(scratch), 'LANG': 'C.UTF-8', 'CI': 'true',
            'npm_config_ignore_scripts': 'true', 'NPM_CONFIG_IGNORE_SCRIPTS': 'true',
            'npm_config_userconfig': '/dev/null', 'npm_config_globalconfig': '/dev/null'}


def source_identity(root):
    env = {'PATH': '/usr/bin:/bin', 'HOME': '/nonexistent',
           'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': '/dev/null'}

    def git(*args):
        return subprocess.check_output(['git', *args], cwd=root, env=env, timeout=20)

    sha = git('rev-parse', 'HEAD').decode().strip()
    tree = git('rev-parse', 'HEAD^{tree}').decode().strip()
    manifest = parse_json(git('show', 'HEAD:' + MANIFEST))
    # The smoke fixture uses committed runtime helper text, not an installed
    # package or a mutable checkout override. Nothing is built for publication.
    runtime = {}
    total = 0
    for entry in git('ls-tree', '-rz', 'HEAD', '--', 'packages/runtime/src').split(b'\0'):
        if not entry:
            continue
        header, raw_path = entry.split(b'\t', 1)
        mode, kind, oid = header.decode().split()
        path = raw_path.decode()
        check(mode in ['100644', '100755'] and kind == 'blob'
              and path.startswith('packages/runtime/src/')
              and all(part not in ['', '.', '..'] for part in path.split('/')),
              'Committed runtime support entry invalid')
        body = git('cat-file', 'blob', oid)
        total += len(body)
        check(len(body) <= 1024 ** 2 and total <= 4 * 1024 ** 2 and len(runtime) < 100,
              'Committed runtime support exceeds bound')
        runtime[path.removeprefix('packages/runtime/')] = body
    check('src/modules/module-helper.ts' in runtime, 'Committed runtime support missing')
    manifests, wrappers = {}, {}
    for name, spec in HOST_TOOLS.items():
        manifests[name] = parse_json(git('show', 'HEAD:' + spec['manifest']))
        if 'wrapper' in spec:
            wrappers[name] = parse_json(git('show', 'HEAD:' + spec['wrapper']))
    return {'sha': sha, 'tree': tree, 'manifest': manifest, 'runtime': runtime,
            'manifests': manifests, 'wrappers': wrappers}


def selected_input(selection, source, package=NAME):
    spec = specification(package)
    key = 'npm:' + package
    manifest = source['manifest'] if package == NAME else source.get('manifests', {}).get(package, {})
    check(isinstance(selection, dict) and selection.get('selectionPolicy') == 'registry-exact-version-v2'
          and selection.get('chain') == 'release' and selection.get('profile') == 'release',
          'Registry host requires the exact-version release policy')
    check(selection.get('sourceSHA') == source['sha'] and selection.get('sourceTree') == source['tree']
          and re.fullmatch('[a-f0-9]{40}', source['sha'])
          and re.fullmatch('[a-f0-9]{40}', source['tree']), 'Registry host source differs')
    check(selection.get('planHash') == digest({key: value for key, value in selection.items()
                                               if key != 'planHash'}), 'Registry host selection digest differs')
    row = selection.get('registryBuildInputs', {}).get(key)
    retained = selection.get('registryRetained', {}).get(key)
    check(isinstance(row, dict) and isinstance(retained, dict)
          and row == {**retained, 'origin': 'registry-present'},
          'Registry host input differs from retained published version')
    expected = {'kind': 'npm', 'name': package, 'owner': spec['owner'],
                'platform': PLATFORM, 'manifestPath': spec['manifest'], 'status': 'published',
                'evidenceKind': 'registry-metadata'}
    check(all(row.get(key) == value for key, value in expected.items()),
          'Registry host identity differs')
    version = row.get('version')
    check(isinstance(version, str) and re.fullmatch(r'(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)', version)
          and manifest.get('name') == package and manifest.get('version') == version,
          'Registry host version differs from committed source declaration')
    if package != NAME:
        wrapper = source.get('wrappers', {}).get(package, {})
        check(wrapper.get('name') == spec['owner']
              and wrapper.get('optionalDependencies', {}).get(package) == version
              and manifest.get('main') == spec['native'] and manifest.get('os') == ['linux']
              and manifest.get('cpu') == ['x64'] and manifest.get('libc') == ['glibc']
              and not manifest.get('scripts'), 'Committed plugin manifest or wrapper differs')
    check(key not in selection.get('build', []) and key not in selection.get('publish', [])
          and all(cell.get('package') != key for field in ['nativeBuildCells', 'preparationNativeCells']
                  for cell in selection.get(field, [])), 'Published host cannot also be rebuilt')
    if 'archiveSha256' in row:
        check(isinstance(row['archiveSha256'], str) and re.fullmatch('[a-f0-9]{64}', row['archiveSha256']),
              'Registry host optional archive digest invalid')
    check(isinstance(row.get('integrity'), str), 'Registry host SHA512 integrity missing')
    try:
        encoded = row.get('integrity', '').removeprefix('sha512-')
        integrity = base64.b64decode(encoded, validate=True)
        check(row.get('integrity') == 'sha512-' + base64.b64encode(integrity).decode()
              and len(integrity) == 64, 'Registry host SHA512 integrity missing')
    except (ValueError, TypeError):
        raise ValueError('Registry host SHA512 integrity invalid') from None
    safe_url(row.get('tarballURL'))
    if 'archiveBytes' in row:
        check(type(row['archiveBytes']) is int and 0 < row['archiveBytes'] <= archive.MAX_COMPRESSED,
              'Registry host archive size invalid')
    return row


def validated_binary(row, read=public_read, package=NAME):
    spec = specification(package)
    native_name = spec['native']
    url = 'https://registry.npmjs.org/' + urllib.parse.quote(package, safe='') + '/' + row['version']
    metadata_body = read(url, MAX_JSON)
    check(type(metadata_body) is bytes and 0 < len(metadata_body) <= MAX_JSON,
          'Registry host metadata exceeds bound')
    metadata = parse_json(metadata_body)
    check(isinstance(metadata, dict) and metadata.get('name') == package and metadata.get('version') == row['version'],
          'Registry host metadata identity differs')
    dist = metadata.get('dist', {})
    check(isinstance(dist, dict), 'Registry host distribution metadata invalid')
    check(safe_url(dist.get('tarball')) == row['tarballURL'], 'Registry host tarball changed')
    check(isinstance(dist.get('integrity'), str) and row['integrity'] in dist['integrity'].split(),
          'Registry host metadata integrity changed')
    body = read(row['tarballURL'], archive.MAX_COMPRESSED)
    check(type(body) is bytes and 0 < len(body) <= archive.MAX_COMPRESSED,
          'Registry host archive exceeds bound')
    check('archiveBytes' not in row or len(body) == row['archiveBytes'], 'Registry host archive size changed')
    actual_sha256 = hashlib.sha256(body).hexdigest()
    check('sha512-' + base64.b64encode(hashlib.sha512(body).digest()).decode() == row['integrity']
          and ('archiveSha256' not in row or actual_sha256 == row['archiveSha256']),
          'Registry host archive transfer integrity differs')
    # Selection only reads metadata. Actual archive hashes are measured here
    # because this one package will execute as a host tool, never compared with
    # newly built source bytes or claimed as a native build from this commit.
    manifest = parse_json(archive.archive_manifest(body))
    check(isinstance(manifest, dict) and manifest.get('name') == package and manifest.get('version') == row['version']
          and manifest.get('main') == native_name and manifest.get('os') == ['linux']
          and manifest.get('cpu') == ['x64'] and manifest.get('libc') == ['glibc']
          and not manifest.get('scripts'), 'Registry host package contract differs')
    allowed = {'package', 'package/package.json', 'package/' + native_name, 'package/README.md',
               'package/LICENSE', 'package/THIRD_PARTY_NOTICES.md',
               'package/license-components.json', 'package/licenses'}
    binary = None
    with tarfile.open(fileobj=io.BytesIO(body), mode='r:gz') as tar:
        for member in tar:
            name = archive.canonical_path(member.name, member.isdir())
            check((member.isfile() or member.isdir()) and not member.linkname
                  and (name in allowed or name.startswith('package/licenses/')),
                  'Unexpected registry host archive member')
            if name == 'package/' + native_name:
                check(binary is None and member.isfile() and 0 < member.size <= archive.MAX_COMPRESSED,
                      'Registry host native entry invalid')
                binary = tar.extractfile(member).read()
                check(len(binary) == member.size, 'Registry host native size differs')
    check(binary is not None and len(binary) >= 64 and binary[:6] == b'\x7fELF\x02\x01'
          and int.from_bytes(binary[16:18], 'little') == 3
          and int.from_bytes(binary[18:20], 'little') == 62,
          'Registry host is not an x64 ELF shared library')
    return binary, len(body), actual_sha256


def node_smoke(node, binary, source, scratch, *, plugin=None, package=NAME):
    check(Path(node).is_absolute() and Path(node).is_file(), 'Absolute Node executable required')
    environment = clean_environment(node, scratch)
    version = subprocess.run([str(node), '--version'], cwd=scratch, env=environment,
                             capture_output=True, timeout=10)
    check(version.returncode == 0 and version.stdout.strip() == NODE_VERSION.encode(),
          'Registry host requires Node v22.20.0; no rebuild fallback')
    runtime = scratch / 'runtime'
    for name, body in source['runtime'].items():
        target = runtime / name
        target.parent.mkdir(parents=True, exist_ok=True)
        with target.open('xb') as file:
            file.write(body)
    script = Path(__file__).with_name('release-registry-host-smoke.mjs')
    arguments = [str(node), str(script), str(binary), str(runtime)]
    if plugin is not None:
        specification(package)
        arguments.extend([str(plugin), package])
    result = subprocess.run(arguments, cwd=scratch,
                            env=environment, capture_output=True, timeout=45)
    check(result.returncode == 0, 'Registry host Node/API/compiler smoke failed; no rebuild fallback')
    check(len(result.stdout) <= MAX_JSON and len(result.stderr) <= MAX_JSON,
          'Registry host smoke output exceeds bound')
    rows = [line.removeprefix(b'FARM_REGISTRY_HOST_SMOKE=') for line in result.stdout.splitlines()
            if line.startswith(b'FARM_REGISTRY_HOST_SMOKE=')]
    check(len(rows) == 1, 'Registry host smoke receipt missing or duplicated')
    return parse_json(rows[0])


def validate_smoke(result, package=NAME):
    check(isinstance(result, dict) and result.get('node') == NODE_VERSION
          and result.get('platform') == 'linux' and result.get('arch') == 'x64'
          and result.get('libc') == 'glibc'
          and isinstance(result.get('napi'), str) and re.fullmatch('[1-9][0-9]*', result['napi'])
          and result.get('requiredExports') == EXPORTS
          and all(result.get(key) is True for key in ['resolver', 'compile', 'tsTransformed', 'outputExecuted'])
          and type(result.get('outputBytes')) is int and 0 < result['outputBytes'] <= MAX_JSON
          and isinstance(result.get('outputSha256'), str) and re.fullmatch('[a-f0-9]{64}', result['outputSha256']),
          'Registry host smoke contract differs')
    if package != NAME:
        check(result.get('plugin') == package and result.get('rustPluginLoaded') is True
              and result.get('pluginFixture') is True, 'Registry plugin smoke contract differs')
    return result


@contextmanager
def parent_directory(root, relative, create=False, absent=True):
    """Walk fixed parents using directory FDs; never follow a destination link."""
    root = Path(root).absolute()
    check(root == root.resolve(strict=True), 'Registry host repository path redirects')
    parts = relative.split('/')
    check(all(part not in ['', '.', '..'] for part in parts), 'Registry host output path invalid')
    descriptor = os.open(root, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        for part in parts[:-1]:
            if create:
                try:
                    os.mkdir(part, 0o755, dir_fd=descriptor)
                except FileExistsError:
                    pass
            following = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=descriptor)
            os.close(descriptor)
            descriptor = following
        try:
            os.stat(parts[-1], dir_fd=descriptor, follow_symlinks=False)
        except FileNotFoundError:
            pass
        else:
            if absent:
                raise FileExistsError('Registry host destination already exists')
        yield descriptor, parts[-1]
    finally:
        os.close(descriptor)


def write_new(parent, name, body):
    descriptor = os.open(name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                         0o444, dir_fd=parent)
    with os.fdopen(descriptor, 'wb') as file:
        file.write(body)
        file.flush()
        check(stat.S_ISREG(os.fstat(file.fileno()).st_mode), 'Registry host output is not regular')


def read_fixed_file(root, relative, limit):
    """Read a fixed repository file without accepting symlink parents or leaves."""
    with parent_directory(root, relative, absent=False) as (parent, name):
        descriptor = os.open(name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=parent)
        with os.fdopen(descriptor, 'rb') as file:
            info = os.fstat(file.fileno())
            check(stat.S_ISREG(info.st_mode) and 0 < info.st_size <= limit,
                  'Registry host input must be a bounded regular file')
            body = file.read(limit + 1)
            check(len(body) == info.st_size and len(body) <= limit, 'Registry host input size changed')
            return body


def plugin_workspace(root, package, source):
    spec = specification(package)
    manifest = parse_json(read_fixed_file(root, spec['manifest'], MAX_JSON))
    check(manifest == source['manifests'][package], 'Registry plugin workspace manifest differs from commit')
    destination = root / spec['destination']
    target = root / spec['link']
    # pnpm's optional dependency must point to this precise workspace package.
    # It is observed, never traversed for a write or used as a trust source.
    check(target.resolve(strict=True) == destination.parent,
          'Registry plugin wrapper resolution differs from workspace platform')
    return {'path': spec['link'], 'destination': str(destination.parent.relative_to(root)),
            'workspacePlatformVerified': True}


def compiler_material(selection, root, source):
    binary = read_fixed_file(root, DESTINATION, archive.MAX_COMPRESSED)
    sha256 = hashlib.sha256(binary).hexdigest()
    if KEY in selection.get('registryBuildInputs', {}):
        row = selected_input(selection, source)
        report = parse_json(read_fixed_file(root, OUTPUT, MAX_JSON))
        expected = {'schemaVersion': 1, 'kind': 'registry-native-host-build-material',
                    'origin': 'registry-present', 'sourceSHA': source['sha'], 'sourceTree': source['tree'],
                    'planHash': selection['planHash'], 'name': NAME, 'version': row['version'],
                    'platform': PLATFORM, 'integrity': row['integrity'], 'tarballURL': row['tarballURL'],
                    'destination': DESTINATION, 'binaryBytes': len(binary), 'binarySha256': sha256,
                    'evidenceKind': 'registry-archive', 'registryBytesVerified': True,
                    'sourceNativeArtifact': False, 'publicationCandidate': False}
        check(isinstance(report, dict) and all(type(report.get(key)) is type(value)
              and report.get(key) == value for key, value in expected.items()),
              'Registry plugin core host material differs')
        validate_smoke(report.get('smoke'))
        origin = 'registry-present'
    else:
        cells = [cell for field in ['nativeBuildCells', 'preparationNativeCells']
                 for cell in selection.get(field, []) if cell.get('package') == KEY]
        expected_cell = {'package': KEY, 'owner': '@farmfe/core', 'platform': PLATFORM}
        # A missing cell may serve both publication and JS host preparation.
        check(bool(cells) and all(cell == expected_cell for cell in cells)
              and selection.get('registrySnapshot', {}).get('packages', {}).get(KEY, {}).get('status') == 'missing'
              and KEY not in selection.get('registryRetained', {}),
              'Registry plugin requires selected source core or verified registry core; no rebuild fallback')
        origin = 'source-build'
    check(len(binary) >= 64 and binary[:6] == b'\x7fELF\x02\x01'
          and int.from_bytes(binary[16:18], 'little') == 3
          and int.from_bytes(binary[18:20], 'little') == 62,
          'Registry plugin core is not an x64 ELF shared library')
    return binary, {'name': NAME, 'origin': origin, 'destination': DESTINATION,
                    'binarySha256': sha256, 'binaryBytes': len(binary)}


def materialize(selection, root, destination, output, node, *, source_reader=source_identity,
                read=public_read, smoke=node_smoke, package=NAME):
    spec = specification(package)
    root = Path(root).absolute()
    check(Path(destination).absolute() == root / spec['destination']
          and Path(output).absolute() == root / spec['output'], 'Registry host output must use fixed paths')
    source = source_reader(root)
    row = selected_input(selection, source, package)
    with parent_directory(root, spec['destination']) as (native_parent, native_name), \
            parent_directory(root, spec['output'], create=True) as (report_parent, report_name):
        wrapper, compiler = None, None
        if package != NAME:
            wrapper = plugin_workspace(root, package, source)
            core_binary, compiler = compiler_material(selection, root, source)
        binary, archive_bytes, archive_sha256 = validated_binary(row, read, package)
        with tempfile.TemporaryDirectory(prefix='farm-registry-host-') as directory:
            scratch = Path(directory)
            native = scratch / spec['native']
            with native.open('xb') as file:
                file.write(binary)
            native.chmod(0o444)
            if package == NAME:
                proof = validate_smoke(smoke(node, native, source, scratch))
            else:
                core_native = scratch / NATIVE
                with core_native.open('xb') as file:
                    file.write(core_binary)
                core_native.chmod(0o444)
                proof = validate_smoke(smoke(node, core_native, source, scratch,
                                             plugin=native, package=package), package)
        report = {'schemaVersion': 1, 'kind': 'registry-native-host-build-material',
                  'origin': 'registry-present', 'sourceSHA': source['sha'], 'sourceTree': source['tree'],
                  'planHash': selection['planHash'], 'name': package, 'version': row['version'],
                  'platform': PLATFORM, 'integrity': row['integrity'], 'tarballURL': row['tarballURL'],
                  'archiveSha256': archive_sha256, 'archiveBytes': archive_bytes,
                  'evidenceKind': 'registry-archive', 'registryBytesVerified': True,
                  'binarySha256': hashlib.sha256(binary).hexdigest(), 'binaryBytes': len(binary),
                  'destination': spec['destination'], 'smoke': proof,
                  'smokeSupport': {'kind': 'committed-runtime-source', 'sourceSHA': source['sha']},
                  'sourceNativeArtifact': False, 'publicationCandidate': False,
                  'scope': 'Exact registry build tool; load and fixture compilation only. No Farm ABI or source-native provenance claim.'}
        if package != NAME:
            # Recheck mutable build inputs after the isolated smoke, before any
            # output is accepted. No registry material becomes a source artifact.
            check(plugin_workspace(root, package, source) == wrapper, 'Registry plugin wrapper changed')
            check(compiler_material(selection, root, source)[1] == compiler, 'Registry plugin core changed')
            report.update(owner=spec['owner'], compiler=compiler, coreBinarySha256=compiler['binarySha256'],
                          wrapperResolution=wrapper,
                          scope='Exact registry build tool; Farm plugin loaded and fixture compiled with the recorded core. No source-native provenance claim.')
        write_new(native_parent, native_name, binary)
        write_new(report_parent, report_name, (json.dumps(report, indent=2) + '\n').encode())
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['selection', 'output', 'destination', 'node']:
        parser.add_argument('--' + name, required=True)
    parser.add_argument('--package', default=NAME, choices=sorted(HOST_TOOLS))
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    with Path(args.selection).open('rb') as file:
        body = file.read(32 * 1024 ** 2 + 1)
    check(len(body) <= 32 * 1024 ** 2, 'Registry host selection exceeds bound')
    selection = parse_json(body)
    check(re.fullmatch('[a-f0-9]{64}', os.environ.get('FARM_RELEASE_PLAN_HASH', ''))
          and selection.get('planHash') == os.environ['FARM_RELEASE_PLAN_HASH'],
          'Registry host selection differs from trusted plan output')
    report = materialize(selection, root, Path(args.destination).absolute(),
                         Path(args.output).absolute(), args.node, package=args.package)
    print(json.dumps({'kind': report['kind'], 'name': report['name'], 'version': report['version'],
                      'destination': report['destination'], 'sourceNativeArtifact': False}))


if __name__ == '__main__':
    main()
