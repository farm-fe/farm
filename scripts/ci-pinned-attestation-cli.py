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

VERSION='2.103.3'
ARCHIVE='gh_'+VERSION+'_linux_amd64.tar.gz'
SHA256='4c806978ffce1cec70853302a56d70ac6da4f19bae7adca4e1a8ab00633dd91d'
SIZE=15031972
URL='https://github.com/cli/cli/releases/download/v'+VERSION+'/'+ARCHIVE
MEMBER=ARCHIVE.removesuffix('.tar.gz')+'/bin/gh'


def unpack_verified_binary(body, digest, member):
    """Pure archive boundary; test fixtures are not the production pinned asset."""
    if not isinstance(body,bytes) or not 0<len(body)<=32*1024*1024 or hashlib.sha256(body).hexdigest()!=digest:
        raise ValueError('Reviewed verifier archive digest differs')
    with tarfile.open(fileobj=io.BytesIO(body),mode='r:gz') as archive:
        rows=[row for row in archive.getmembers() if row.name==member]
        if len(rows)!=1 or not rows[0].isreg() or not 0<rows[0].size<=128*1024*1024:
            raise ValueError('Verifier executable archive member differs')
        binary=archive.extractfile(rows[0]).read()
        if len(binary)!=rows[0].size:
            raise ValueError('Verifier executable bytes incomplete')
        return binary


def verify_on_host(manifest, bundle, expected):
    if (os.environ.get('GITHUB_ACTIONS')!='true' or os.environ.get('RUNNER_ENVIRONMENT')!='github-hosted'
            or sys.platform!='linux' or platform.machine() not in ['x86_64','AMD64']):
        raise PermissionError('Production signature verifier provisioning requires the hosted Linux x64 consumer')
    # Public GET only; no existing token or authentication header is forwarded.
    with urllib.request.urlopen(urllib.request.Request(URL,method='GET'),timeout=60) as response:
        body=response.read(32*1024*1024+1)
    if len(body)!=SIZE:
        raise ValueError('Reviewed verifier archive size differs')
    binary=unpack_verified_binary(body,SHA256,MEMBER)
    spec=importlib.util.spec_from_file_location('pinned_binding_verifier',Path(__file__).with_name('ci-attested-bindings.py'))
    policy=importlib.util.module_from_spec(spec);spec.loader.exec_module(policy)
    with tempfile.TemporaryDirectory(prefix='farm-pinned-verifier-') as directory:
        cli=Path(directory)/'gh';cli.write_bytes(binary);cli.chmod(stat.S_IRUSR|stat.S_IWUSR|stat.S_IXUSR)
        return policy.verify_with_pinned_cli(cli,hashlib.sha256(binary).hexdigest(),manifest,bundle,expected)
