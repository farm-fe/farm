import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import { serveBuild, verifyBuiltFarm, correctnessFailureReceipt } from './performance-correctness-worker.mjs';

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
test('failed Farm browser flow closes the owned context and rejects evidence', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-perf-browser-'));
  let closed = 0;
  try {

    const browser = { newContext: async () => ({ close: async () => { closed++; }, newPage: async () => ({ setDefaultTimeout() {}, on() {}, off() {}, goto: async () => { throw new Error('Real page failure'); } }) }) };
    await assert.rejects(verifyBuiltFarm(root, browser, async page => { await page.goto('http://localhost'); }), /Real page failure/);
    assert.equal(closed, 1);
  } finally { await fs.rm(root, { recursive: true }); }
});
test('Farm-only preview needs no second output and closes its context on success and failure', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-perf-flow-'));
  let closed = 0, visits = 0;
  const browser = { newContext: async () => ({ close: async () => { closed++; }, newPage: async () => ({ setDefaultTimeout() {} }) }) };
  try {
    const result = await verifyBuiltFarm(root, browser, async () => { visits++; return { functional: true }; });
    assert.equal(result.measured, false); assert.equal(result.status, 'complete');
    assert.equal(result.result.functional, true); assert.equal(visits, 1); assert.equal(closed, 1);
    await assert.rejects(verifyBuiltFarm(root, browser, async () => ({ functional: false })), /Farm browser correctness failed/);
    assert.equal(closed, 2);
  } finally { await fs.rm(root, { recursive: true }); }
});

test('context/preview cleanup failures cannot replace the original browser diagnostics', async () => {
  const original = Object.assign(new Error('Original runtime error'), { browserEvidence: { url: 'http://fixture/#components' } });
  const browser = { newContext: async () => ({ close: async () => { throw new Error('Context close failed'); }, newPage: async () => ({ setDefaultTimeout() {} }) }) };
  const serve = async () => ({ url: 'http://fixture', close: async () => { throw new Error('Preview close failed'); } });
  await assert.rejects(verifyBuiltFarm('/fixture', browser, async () => { throw original; }, { serve }), error => {
    assert.equal(error, original); assert.equal(error.cleanupErrors.length, 2);
    const receipt = correctnessFailureReceipt(error);
    assert.equal(receipt.error, 'Original runtime error'); assert.equal(receipt.diagnostics.url, 'http://fixture/#components');
    assert.equal(receipt.cleanupErrors.length, 2); return true;
  });
  await assert.rejects(verifyBuiltFarm('/fixture', browser, async () => ({ functional: true }), { serve }), /cleanup failed/);
});
