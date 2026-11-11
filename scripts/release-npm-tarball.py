"""Fail-closed inspection of the original npm archive before OIDC is exposed.

No extraction, npm invocation, normalization or repacking is performed. Call
validate for EVERY selected archive before starting any npm publication, and
again at the exact-byte upload boundary. Rejected metadata must be corrected
in source and rebuilt, never patched into an already frozen archive.
"""
# cspell:words npmjs USTAR PAX SCHILY casefold uncompressed
import gzip
import hashlib
import io
import json
import re
import tarfile
import unicodedata
import zlib

MAX_COMPRESSED = 100 * 1024 ** 2
MAX_EXPANDED = 200 * 1024 ** 2
MAX_MANIFEST = 1024 ** 2
MAX_MEMBERS = 20000
MAX_PAX = 64 * 1024
MANIFEST_PATH = 'package/package.json'
REGISTRY = 'https://registry.npmjs.org/'
REPOSITORIES = frozenset({
    'https://github.com/farm-fe/farm',
    'https://github.com/farm-fe/farm.git',
    'git+https://github.com/farm-fe/farm',
    'git+https://github.com/farm-fe/farm.git',
})


def check(condition, message):
    if not condition:
        raise ValueError(message)


class BoundedGzip:
    def __init__(self, body):
        self.stream = gzip.GzipFile(fileobj=io.BytesIO(body), mode='rb')
        self.total = 0

    def read(self, size):
        data = self.stream.read(min(size, MAX_EXPANDED - self.total + 1))
        self.total += len(data)
        check(self.total <= MAX_EXPANDED, 'Expanded npm archive exceeds bound')
        return data

    def exact(self, size):
        data = self.read(size)
        check(len(data) == size, 'Truncated npm archive')
        return data

    def payload(self, size, keep=False):
        check(0 <= size <= MAX_EXPANDED - self.total, 'Expanded npm member exceeds bound')
        chunks = []
        while size:
            data = self.exact(min(size, 65536))
            size -= len(data)
            if keep:
                chunks.append(data)
        return b''.join(chunks)


def canonical_path(name, directory=False):
    check(isinstance(name, str) and 0 < len(name.encode('utf-8')) <= 4096,
          'Invalid npm archive path')
    check(not any(ord(char) < 32 or ord(char) == 127 or char in '\\:*?"<>|' for char in name)
          and unicodedata.normalize('NFC', name) == name,
          'Noncanonical npm archive path')
    if directory and name.endswith('/'):
        name = name[:-1]
    parts = name.split('/')
    check(parts[0] == 'package' and all(part not in ['', '.', '..'] for part in parts)
          and (directory or len(parts) > 1), 'External or noncanonical npm archive path')
    check(all(not part.endswith((' ', '.')) and not re.fullmatch(
        r'(?:con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\..*)?', part, re.I) for part in parts),
          'Nonportable npm archive path')
    return name


def pax_fields(body):
    # npm/node-tar parses one line at a time. Reject multiline values and
    # duplicate keys instead of allowing parsers to select different records.
    result = {}
    for line in body.splitlines(keepends=True):
        match = re.fullmatch(rb'([1-9][0-9]{0,5}) ([^=\n]+)=([^\n]*)\n', line)
        check(match is not None and int(match[1]) == len(line), 'Malformed npm PAX record')
        key, value = match[2].decode('utf-8'), match[3].decode('utf-8')
        check(key not in result, 'Duplicate npm PAX key')
        check(key in {'path', 'size', 'linkpath', 'mtime', 'atime', 'ctime', 'uid', 'gid',
                      'uname', 'gname', 'SCHILY.dev', 'SCHILY.ino', 'SCHILY.nlink'},
              'Unsupported npm PAX override')
        check(not any(ord(char) < 32 or ord(char) == 127 for char in value), 'Invalid npm PAX value')
        result[key] = value
    check(bool(result), 'Empty npm PAX record')
    return result


def archive_manifest(body):
    stream = BoundedGzip(body)
    manifest = None
    seen, files, parents = set(), set(), set()
    pending = None
    count = 0
    try:
        while True:
            header = stream.exact(512)
            if header == bytes(512):
                check(pending is None, 'Orphan npm PAX header')
                check(stream.exact(512) == bytes(512), 'Ambiguous npm archive end')
                # Consume padding and gzip trailers too: do not overlook a
                # second tar archive/member after the first end marker.
                while True:
                    tail = stream.read(65536)
                    if not tail:
                        break
                    check(not any(tail), 'Unexpected trailing npm archive data')
                check(stream.total % 512 == 0, 'Unaligned npm archive padding')
                break
            count += 1
            check(count <= MAX_MEMBERS, 'Npm archive member count exceeds bound')
            check(header[257:265] == b'ustar\x0000', 'Unsupported npm tar format')
            # tarfile checks header checksums but accepts signed checksums too;
            # npm requires the unsigned checksum, so enforce it explicitly.
            checksum = header[148:156].strip(b'\x00 ')
            check(bool(re.fullmatch(rb'[0-7]+', checksum))
                  and int(checksum, 8) == sum(header[:148]) + 256 + sum(header[156:]),
                  'Invalid npm tar checksum')
            member = tarfile.TarInfo.frombuf(header, 'utf-8', 'strict')
            check(member.type in [tarfile.REGTYPE, tarfile.AREGTYPE, tarfile.DIRTYPE, tarfile.XHDTYPE],
                  'Unsupported npm special member')
            check(member.size >= 0, 'Invalid npm member size')
            if member.type == tarfile.XHDTYPE:
                check(pending is None, 'Stacked npm PAX headers')
                check(0 < member.size <= MAX_PAX, 'Npm PAX header exceeds bound')
                pending = pax_fields(stream.payload(member.size, keep=True))
                padding = stream.exact((-member.size) % 512)
                check(not any(padding), 'Nonzero npm member padding')
                continue
            directory = member.type == tarfile.DIRTYPE
            raw_name = canonical_path(member.name, directory)
            check(not member.linkname, 'Npm member link target forbidden')
            name = raw_name
            if pending is not None:
                if 'path' in pending:
                    # npm/node-tar ReadEntry reapplies the complete PAX path
                    # after Header handles USTAR prefixes, just like tarfile.
                    name = canonical_path(pending['path'], directory)
                if 'size' in pending:
                    check(bool(re.fullmatch(r'(?:0|[1-9][0-9]*)', pending['size']))
                          and int(pending['size']) == member.size, 'Conflicting npm PAX size')
                check(not pending.get('linkpath'), 'Npm PAX link target forbidden')
                pending = None
            key = name.casefold()
            check(key not in seen, 'Duplicate npm archive path')
            seen.add(key)
            parts = key.split('/')
            ancestry = {'/'.join(parts[:i]) for i in range(1, len(parts))}
            check(not ancestry.intersection(files) and (directory or key not in parents),
                  'Conflicting npm archive file and directory')
            parents.update(ancestry)
            if directory:
                check(member.size == 0, 'Npm directory has payload')
            else:
                check(not member.name.endswith('/'), 'Ambiguous npm regular directory')
                files.add(key)
            is_manifest = name == MANIFEST_PATH
            if key == MANIFEST_PATH:
                check(is_manifest and not directory, 'Npm package manifest path is not exact')
                check(0 < member.size <= MAX_MANIFEST, 'Npm package manifest exceeds bound')
            data = stream.payload(member.size, keep=is_manifest)
            if is_manifest:
                manifest = data
            check(not any(stream.exact((-member.size) % 512)), 'Nonzero npm member padding')
        check(manifest is not None, 'Exact npm package manifest missing')
        return manifest
    except (OSError, EOFError, UnicodeError, tarfile.TarError, zlib.error):
        raise ValueError('Invalid compressed npm archive') from None
    finally:
        stream.stream.close()


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        check(key not in result, 'Duplicate npm package JSON key')
        result[key] = value
    return result


def invalid_constant(_):
    raise ValueError('Nonstandard npm package JSON value')


def validate_manifest(row, manifest):
    check(isinstance(manifest, dict), 'Npm package manifest must be an object')
    check(manifest.get('name') == row['name'] and manifest.get('version') == row['version'],
          'Frozen npm package identity differs')
    check('private' not in manifest or manifest['private'] is False, 'Private npm package rejected')
    # @npmcli/package-json.prepare normalizes repositories[0] over repository.
    check('repositories' not in manifest, 'Legacy npm repositories override forbidden')
    check('tag' not in manifest, 'Top-level npm tag override forbidden')
    repository = manifest.get('repository')
    if isinstance(repository, dict):
        check(repository.get('type', 'git') == 'git', 'Unsupported npm repository type')
        repository = repository.get('url')
    check(isinstance(repository, str) and repository in REPOSITORIES,
          'Npm provenance repository must identify farm-fe/farm using a canonical root URL')
    if 'publishConfig' in manifest:
        config = manifest['publishConfig']
        check(isinstance(config, dict), 'Npm publishConfig must be an object')
        # npm accepts arbitrary config keys here, including scoped registries,
        # auth, proxy and provenance-file. A denylist is not sufficient.
        check(set(config) <= {'access', 'registry', 'provenance', 'tag'},
              'Unsupported npm publishConfig override')
        check('access' not in config or config['access'] == 'public', 'Npm publishConfig access differs')
        check('registry' not in config or config['registry'] in [REGISTRY, REGISTRY[:-1]],
              'Npm publishConfig registry differs')
        check('provenance' not in config or config['provenance'] is True,
              'Npm publishConfig provenance differs')
        check('tag' not in config or config['tag'] == 'farm-release-' + row['version'],
              'Npm publishConfig tag differs')
    return manifest


def validate(row, body):
    """Return checked metadata, or raise ValueError without echoing its values."""
    check(isinstance(row, dict) and row.get('kind') == 'npm', 'Expected frozen npm record')
    check(isinstance(row.get('name'), str) and len(row['name']) <= 214
          and re.fullmatch(r'(?:@[a-z0-9][a-z0-9._-]*/)?[a-z0-9][a-z0-9._-]*', row['name']),
          'Invalid frozen npm name')
    check(isinstance(row.get('version'), str) and len(row['version']) <= 60
          and re.fullmatch(r'(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)', row['version']),
          'Invalid frozen npm stable version')
    check(all(int(part) <= 9007199254740991 for part in row['version'].split('.')),
          'Frozen npm version exceeds SemVer integer bound')
    check(row['name'] not in ['node_modules', 'favicon.ico'], 'Invalid frozen npm name')
    check(type(body) is bytes and 0 < len(body) <= MAX_COMPRESSED, 'Compressed npm archive exceeds bound')
    check(hashlib.sha256(body).hexdigest() == row.get('archiveSha256'), 'Frozen npm archive checksum differs')
    raw = archive_manifest(body)
    try:
        manifest = json.loads(raw.decode('utf-8'), object_pairs_hook=unique_object, parse_constant=invalid_constant)
    except (UnicodeError, json.JSONDecodeError, RecursionError):
        raise ValueError('Invalid npm package JSON') from None
    return validate_manifest(row, manifest)
