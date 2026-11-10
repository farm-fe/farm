"""Temporary reviewed public CLI verifier; no login, install, signing or upload."""
# cspell:words importlib sigstore
import hashlib
import importlib.util
import io
import os
from pathlib import Path
import platform
import stat
import sys
import tarfile
import tempfile
import urllib.request
import zipfile

VERSION='2.103.3'
# Reviewed immutable cli/cli v2.103.3 assets. Host, not target ABI, selects the verifier.
ASSETS = {
    ('linux', 'x64'): ('gh_2.103.3_linux_amd64.tar.gz', 15031972, '4c806978ffce1cec70853302a56d70ac6da4f19bae7adca4e1a8ab00633dd91d', 'gh_2.103.3_linux_amd64/bin/gh'),
    ('darwin', 'x64'): ('gh_2.103.3_macOS_amd64.zip', 15702601, 'a2f86418f4350ae6e9ec60d5000bdf7665728436946a97fc27fe50e6ae752682', 'gh_2.103.3_macOS_amd64/bin/gh'),
    ('darwin', 'arm64'): ('gh_2.103.3_macOS_arm64.zip', 14106091, 'd700207eae9ae98fe2b89535c2c087c07823d4628c56364695a5a07ef311c062', 'gh_2.103.3_macOS_arm64/bin/gh'),
    ('win32', 'x64'): ('gh_2.103.3_windows_amd64.zip', 15197187, '98d0cfa794721665d7973dd728c78aff7bd836d58f10f6d7a03568eccb0a22e0', 'bin/gh.exe'),
}


def host_asset(system, machine):
    arch = {'x86_64':'x64', 'amd64':'x64', 'arm64':'arm64', 'aarch64':'arm64'}.get(machine.lower())
    if (system, arch) not in ASSETS:
        raise PermissionError('No reviewed verifier for this hosted platform')
    return ASSETS[(system, arch)]


def unpack_verified_binary(body, digest, member, *, zipped=False):
    """Pure archive boundary; test fixtures are not the production pinned asset."""
    if not isinstance(body,bytes) or not 0<len(body)<=32*1024*1024 or hashlib.sha256(body).hexdigest()!=digest:
        raise ValueError('Reviewed verifier archive digest differs')
    if zipped:
        with zipfile.ZipFile(io.BytesIO(body)) as archive:
            rows=[row for row in archive.infolist() if row.filename==member]
            if (len(rows)!=1 or rows[0].is_dir() or rows[0].flag_bits & 1
                    or stat.S_IFMT(rows[0].external_attr >> 16) not in [0, stat.S_IFREG]
                    or not 0<rows[0].file_size<=128*1024*1024):
                raise ValueError('Verifier executable archive member differs')
            binary=archive.read(rows[0])
            if len(binary)!=rows[0].file_size:
                raise ValueError('Verifier executable bytes incomplete')
            return binary
    with tarfile.open(fileobj=io.BytesIO(body),mode='r:gz') as archive:
        rows=[row for row in archive.getmembers() if row.name==member]
        if len(rows)!=1 or not rows[0].isreg() or not 0<rows[0].size<=128*1024*1024:
            raise ValueError('Verifier executable archive member differs')
        binary=archive.extractfile(rows[0]).read()
        if len(binary)!=rows[0].size:
            raise ValueError('Verifier executable bytes incomplete')
        return binary


def verify_on_host(manifest, bundle, expected):
    if os.environ.get('GITHUB_ACTIONS')!='true' or os.environ.get('RUNNER_ENVIRONMENT')!='github-hosted':
        raise PermissionError('Production signature verifier provisioning requires a GitHub-hosted consumer')
    archive,size,digest,member=host_asset(sys.platform,platform.machine())
    url='https://github.com/cli/cli/releases/download/v'+VERSION+'/'+archive
    # Public GET only; no existing token or authentication header is forwarded.
    with urllib.request.urlopen(urllib.request.Request(url,method='GET'),timeout=60) as response:
        body=response.read(32*1024*1024+1)
    if len(body)!=size:
        raise ValueError('Reviewed verifier archive size differs')
    binary=unpack_verified_binary(body,digest,member,zipped=archive.endswith('.zip'))
    spec=importlib.util.spec_from_file_location('pinned_binding_verifier',Path(__file__).with_name('ci-attested-bindings.py'))
    policy=importlib.util.module_from_spec(spec);spec.loader.exec_module(policy)
    with tempfile.TemporaryDirectory(prefix='farm-pinned-verifier-') as directory:
        cli=Path(directory)/('gh.exe' if sys.platform=='win32' else 'gh');cli.write_bytes(binary);cli.chmod(stat.S_IRUSR|stat.S_IWUSR|stat.S_IXUSR)
        return policy.verify_with_pinned_cli(cli,hashlib.sha256(binary).hexdigest(),manifest,bundle,expected)
