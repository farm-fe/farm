# cspell:ignore Zlockfile
"""Explicit locked workspace fetch before offline Cargo package-input inspection.

Importing this module never performs network access. The CLI requires --fetch;
the existing public-source integration entry calls it only with network opt-in.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import tempfile


def prepare_workspace(root, *, allow_network=False):
    if not allow_network:
        raise ValueError('Explicit Cargo dependency preparation opt-in required')
    root = Path(root).resolve()
    lock = root / 'Cargo.lock'
    original = lock.read_bytes()
    toolchain = (root / 'rust-toolchain.toml').read_text()
    channel = re.search(r'^channel\s*=\s*"(nightly-[0-9-]+)"', toolchain, re.MULTILINE)
    if not channel:
        raise ValueError('Pinned nightly Cargo toolchain required')
    scratch = Path(tempfile.mkdtemp(prefix='farm-cargo-input-preparation-'))
    isolated = scratch / 'Cargo.lock'
    isolated.write_bytes(original)
    command = ['cargo', '+' + channel[1], 'fetch', '--locked', '-Zlockfile-path',
               '--config', 'resolver.lockfile-path=' + json.dumps(str(isolated)),
               '--config', 'build.build-dir=' + json.dumps(str(scratch / 'build')),
               '--config', 'build.target-dir=' + json.dumps(str(scratch / 'target')),
               '--manifest-path', str(root / 'Cargo.toml')]
    try:
        try:
            subprocess.run(command, cwd=root, check=True, timeout=900)
        finally:
            if lock.read_bytes() != original or isolated.read_bytes() != original:
                raise ValueError('Locked Cargo dependency preparation changed lock bytes')
        result = {'status': 'prepared', 'lockSha256': hashlib.sha256(original).hexdigest(),
                  'command': command, 'scope': 'Full pinned workspace, all targets; inherited Cargo home; no compilation or publication'}
    except BaseException:
        print('Cargo dependency preparation diagnostics retained at ' + str(scratch), flush=True)
        raise
    shutil.rmtree(scratch)
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fetch', action='store_true', required=True)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    print(json.dumps(prepare_workspace(args.root, allow_network=args.fetch)))
