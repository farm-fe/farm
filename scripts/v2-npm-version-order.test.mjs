import assert from 'node:assert/strict';
import test from 'node:test';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { compareRegistryLatest, compareVersions } from './v2-npm-version-order.mjs';

test('registry prereleases compare by full SemVer precedence', () => {
  const ordered = ['1.0.0-alpha', '1.0.0-alpha.1', '1.0.0-alpha.beta', '1.0.0-beta',
    '1.0.0-beta.2', '1.0.0-beta.11', '1.0.0-rc.1', '1.0.0'];
  for (let i = 1; i < ordered.length; i++) {
    assert.equal(compareVersions(ordered[i - 1], ordered[i]), -1);
    assert.equal(compareVersions(ordered[i], ordered[i - 1]), 1);
  }
  assert.equal(compareVersions('2.0.0-2', '2.0.0-11'), -1);
  assert.equal(compareVersions('2.0.0-11', '2.0.0-a'), -1);
  assert.equal(compareVersions('2.0.0-rc.1+one', '2.0.0-rc.1+two'), 0);
  assert.equal(compareVersions('2.0.0+one', '2.0.0+two'), 0);
});

test('stable publication target accepts older prerelease latest, never a newer one', () => {
  for (const latest of ['1.0.0-nightly-20241024075304', '2.0.0-rc.1', '2.0.0-beta.1']) {
    assert.equal(compareRegistryLatest(latest, '2.0.0'), -1);
  }
  for (const latest of ['2.1.0-rc.1', '3.0.0-rc.1', '3.0.0']) {
    assert.equal(compareRegistryLatest(latest, '2.0.0'), 1);
  }
  assert.equal(compareRegistryLatest('2.0.0+build.12', '2.0.0'), 0);
  assert.equal(compareRegistryLatest(null, '2.0.0'), null);
});

test('malformed latest and non-canonical or unstable targets fail closed', () => {
  for (const value of ['', 'v2.0.0', '=2.0.0', ' 2.0.0', '2.0.0 ', '01.0.0', '2.0',
    '2.0.0-01', '2.0.0-rc..1', '2.0.0+', '2.0.0+bad_', 2, {}, undefined]) {
    assert.throws(() => compareRegistryLatest(value, '2.0.0'));
  }
  for (const target of ['2.0.0-rc.1', '2.0.0+build', 'v2.0.0', '02.0.0', '', null]) {
    assert.throws(() => compareRegistryLatest(null, target));
  }
});

test('publisher comparison works without any workspace installation', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-version-order-'));
  try {
    for (const name of ['v2-npm-version-order.mjs', 'v2-npm-cli.mjs']) {
      await fs.copyFile(new URL(name, import.meta.url), path.join(root, name));
    }
    const result = spawnSync(process.execPath, [path.join(root, 'v2-npm-version-order.mjs'),
      JSON.stringify(['2.0.0-rc.1', '2.0.0'])], { cwd: root, encoding: 'utf8',
      env: { ...process.env, NODE_PATH: path.join(root, 'nonexistent') } });
    assert.equal(result.status, 0, result.stderr);
    assert.equal(JSON.parse(result.stdout), -1);
  } finally { await fs.rm(root, { recursive: true }); }
});
