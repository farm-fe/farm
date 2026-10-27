import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import { serveBuild, verifyBuiltComparison } from './performance-correctness-worker.mjs';

test('built preview serves JavaScript with MIME and closes its owned server', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-perf-preview-'));
  let server;
  try {
    await fs.writeFile(path.join(root, 'index.html'), '<h1>Catalog</h1>');
    await fs.writeFile(path.join(root, 'app.js'), 'export const answer = 42;');
    server = await serveBuild(root);
    assert.equal((await fetch(server.url)).status, 200);
    assert.equal((await fetch(`${server.url}/app.js`)).headers.get('content-type'), 'text/javascript');
    assert.equal((await fetch(`${server.url}/missing.js`)).status, 404);
    await server.close(); server = null;
  } finally { await server?.close(); await fs.rm(root, { recursive: true }); }
});
test('failed browser flow closes the owned context and rejects comparison', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-perf-browser-'));
  let closed = 0;
  try {
    await fs.mkdir(path.join(root, 'dist-farm'));
    const browser = { newContext: async () => ({ close: async () => { closed++; }, newPage: async () => ({ setDefaultTimeout() {}, on() {}, off() {}, goto: async () => { throw new Error('Real page failure'); } }) }) };
    await assert.rejects(verifyBuiltComparison(root, browser), /Real page failure/);
    assert.equal(closed, 1);
  } finally { await fs.rm(root, { recursive: true }); }
});
test('both tools execute the same functional flow outside measurement; console errors fail', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-perf-flow-'));
  let closed = 0, visits = 0, injectError = false;
  const browser = { newContext: async () => {
    const listeners = new Map();
    const locator = { filter: () => locator, waitFor: async () => {}, selectOption: async () => {}, fill: async () => {}, click: async () => {} };
    return { close: async () => { closed++; }, newPage: async () => ({
      setDefaultTimeout() {}, on: (name, listener) => listeners.set(name, listener), off: name => listeners.delete(name), locator: () => locator,
      goto: async () => { visits++; if (injectError) listeners.get('console')({ type: () => 'error', text: () => 'Broken module' }); }
    }) };
  } };
  try {
    for (const tool of ['farm', 'vite']) await fs.mkdir(path.join(root, `dist-${tool}`));
    const result = await verifyBuiltComparison(root, browser);
    assert.equal(result.measured, false); assert.equal(result.status, 'complete');
    assert.equal(result.results.farm.functional, true); assert.equal(result.results.vite.functional, true);
    assert.equal(visits, 2); assert.equal(closed, 2);
    injectError = true;
    await assert.rejects(verifyBuiltComparison(root, browser), /Console error: Broken module/);
    assert.equal(closed, 3);
  } finally { await fs.rm(root, { recursive: true }); }
});
