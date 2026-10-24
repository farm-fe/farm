import assert from 'node:assert/strict';
import test from 'node:test';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { fingerprintTree, captureSourceProvenance } from '../benchmark/source-provenance.mjs';

test('run provenance identifies the workspace JS artifacts and dependency lock actually used', async () => {
  const report = await captureSourceProvenance();
  assert.ok(report.workspaceRuntime, 'compiled workspace packages must be identified alongside example source');
  const files = new Set(report.workspaceRuntime.files.map(file => file.path));
  for (const required of ['packages/core/dist/index.js', 'packages/cli/bin/farm.mjs', 'packages/runtime/src/module-system.ts', 'js-plugins/sass/build/esm/index.mjs', 'js-plugins/svgr/build/esm/index.mjs', 'js-plugins/tailwindcss/dist/esm/index.mjs', 'pnpm-lock.yaml']) {
    assert.ok(files.has(required), `Missing runtime input: ${required}`);
  }
  assert.match(report.workspaceRuntime.sha256, /^[a-f0-9]{64}$/);
});

test('source digest is ordered, tracks bytes and excludes generated directories', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'commerce-source-'));
  await fs.mkdir(path.join(root, 'apps/dist'), { recursive: true });
  await fs.writeFile(path.join(root, 'apps/a.ts'), 'source');
  await fs.writeFile(path.join(root, 'apps/dist/output.js'), 'generated');
  const first = await fingerprintTree(root, ['apps']);
  assert.deepEqual(first.files.map(file => file.path), ['apps/a.ts']);
  await fs.writeFile(path.join(root, 'apps/dist/output.js'), 'changed output');
  assert.deepEqual(await fingerprintTree(root, ['apps']), first);
  await fs.writeFile(path.join(root, 'apps/a.ts'), 'changed source');
  assert.notEqual((await fingerprintTree(root, ['apps'])).sha256, first.sha256);
  await fs.writeFile(path.join(root, 'b.ts'), 'config');
  assert.deepEqual(await fingerprintTree(root, ['apps', 'b.ts']), await fingerprintTree(root, ['b.ts', 'apps']));
  await assert.rejects(fingerprintTree(root, ['../outside']));
});
