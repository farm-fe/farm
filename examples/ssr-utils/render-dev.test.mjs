import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { renderDev } from './render-dev.mjs';

async function fixture(t, source) {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-ssr-worker-test-'));
  t.after(() => fs.rm(directory, { recursive: true, force: true }));
  const entry = path.join(directory, 'entry.mjs');
  await fs.writeFile(entry, source);
  return { directory, entry };
}

test('reloads the complete async module graph and restores it', async t => {
  const { directory, entry } = await fixture(t, `export default async url => (await import('./page.mjs')).render(url);`);
  const page = path.join(directory, 'page.mjs');
  for (const value of ['original', 'updated', 'original']) {
    await fs.writeFile(page, `export const render = url => '${value}:' + url;`);
    assert.equal(await renderDev(entry, '/orders'), `${value}:/orders`);
  }
});

test('propagates render errors and terminates workers holding active resources', async t => {
  const { entry } = await fixture(t, `setInterval(() => {}, 1000); export default () => { throw new Error('render failed'); };`);
  await assert.rejects(renderDev(entry, '/', { timeoutMs: 1000 }), /render failed/);
});

test('times out and terminates a blocked render worker', async t => {
  const { entry } = await fixture(t, `export default () => { while (true) {} };`);
  await assert.rejects(renderDev(entry, '/', { timeoutMs: 100 }), /timed out/);
});

test('reports worker exit without a render result', async t => {
  const { entry } = await fixture(t, `process.exit(0); export default () => '';`);
  await assert.rejects(renderDev(entry, '/'), /without a render result/);
});
