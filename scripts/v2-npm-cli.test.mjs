import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { resolveNpmCli } from './v2-npm-cli.mjs';

async function fixture(platform) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-bundled-npm-'));
  const node = path.join(root, platform === 'win32' ? 'node.exe' : 'bin/node');
  const npm = path.join(root, platform === 'win32' ? 'node_modules/npm' : 'lib/node_modules/npm');
  await fs.mkdir(path.dirname(node), { recursive: true }); await fs.writeFile(node, 'fixture only');
  await fs.mkdir(path.join(npm, 'bin'), { recursive: true });
  const manifest = { name: 'npm', version: '10.9.3', bin: { npm: 'bin/npm-cli.js' } };
  await fs.writeFile(path.join(npm, 'package.json'), JSON.stringify(manifest));
  const cli = path.join(npm, 'bin/npm-cli.js'); await fs.writeFile(cli, '// fixture only');
  return { root, node, npm, manifest, cli };
}

test('Windows and Unix Node layouts resolve only their bundled npm JavaScript entry', async () => {
  for (const platform of ['win32', 'linux', 'darwin']) {
    const f = await fixture(platform);
    try { assert.equal(resolveNpmCli({ platform, nodeExecutable: f.node }), await fs.realpath(f.cli)); }
    finally { await fs.rm(f.root, { recursive: true }); }
  }
});

test('missing layout, wrong identity, changed bin and escaped realpath fail closed', async () => {
  const f = await fixture('linux');
  try {
    assert.throws(() => resolveNpmCli({ platform: 'win32', nodeExecutable: f.node }));
    for (const manifest of [{ ...f.manifest, name: 'other' }, { ...f.manifest, version: 'unknown' }, { ...f.manifest, bin: { npm: '../other.js' } }]) {
      await fs.writeFile(path.join(f.npm, 'package.json'), JSON.stringify(manifest));
      assert.throws(() => resolveNpmCli({ platform: 'linux', nodeExecutable: f.node }));
    }
    await fs.writeFile(path.join(f.npm, 'package.json'), JSON.stringify(f.manifest));
    const outside = path.join(f.root, 'outside'); await fs.mkdir(outside);
    await fs.writeFile(path.join(outside, 'npm-cli.js'), '// outside package');
    await fs.rm(path.join(f.npm, 'bin'), { recursive: true });
    await fs.symlink(outside, path.join(f.npm, 'bin'), process.platform === 'win32' ? 'junction' : 'dir');
    assert.throws(() => resolveNpmCli({ platform: 'linux', nodeExecutable: f.node }), /stay inside/);
  } finally { await fs.rm(f.root, { recursive: true }); }
});

test('actual CI Node distribution exposes the verified bundled manager without a shell lookup', () => {
  assert.ok(path.isAbsolute(resolveNpmCli()));
});
