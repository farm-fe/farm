import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { assertViteVersion, comparisonConfigs, prepareComparisonFixture, collectComparisonArtifacts } from '../benchmark/vite-comparison.mjs';

test('fixed real Vite oracle and matched deterministic Commerce subset', async () => {
  assert.equal((await assertViteVersion()).version, '5.2.8');
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'commerce-comparison-'));
  try {
    const first = await prepareComparisonFixture(path.join(root, 'a'));
    const second = await prepareComparisonFixture(path.join(root, 'b'));
    assert.equal(first.fingerprint, second.fingerprint);
    assert.equal(first.productCount, 240);
    const { businessQuote } = await import(pathToFileURL(path.join(first.root, 'business-quote.mjs')));
    const request = { price: 1000, stock: 100, quantity: 2, discount: 5, destination: 'local', rules: { reserveStock: 2, maxDiscount: 20 }, tiers: [{ minQuantity: 3, discount: 5 }], shipping: [{ destination: 'local', freeAbove: 10000, cents: 500 }] };
    assert.equal(businessQuote(request).total, 2400);
    assert.equal(businessQuote({ ...request, quantity: 3 }).total, 3200);
    assert.throws(() => businessQuote({ ...request, quantity: 100 }), /Insufficient stock/);
    const configs = comparisonConfigs(first.root);
    assert.equal(configs.farm.compilation.input.index, path.join(first.root, 'index.html'));
    assert.equal(configs.vite.root, first.root);
    assert.equal(configs.vite.build.target, 'es2017');
    assert.equal(configs.farm.compilation.output.targetEnv, 'browser-es2017');
    assert.equal(configs.farm.vitePlugins, undefined);
    assert.equal(configs.farm.compilation.persistentCache, false);
  } finally { await fs.rm(root, { recursive: true, force: true }); }
});

test('artifact accounting emits raw/gzip bytes and content hashes', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'commerce-artifacts-'));
  try {
    await fs.writeFile(path.join(root, 'index.html'), '<html>commerce</html>');
    await fs.writeFile(path.join(root, 'entry.js'), 'console.log("commerce");');
    const result = await collectComparisonArtifacts(root);
    assert.equal(result.files.length, 2);
    assert.equal(result.rawBytes, 45);
    assert.ok(result.gzipBytes > 0);
    assert.match(result.files[0].sha256, /^[a-f0-9]{64}$/);
  } finally { await fs.rm(root, { recursive: true, force: true }); }
});
