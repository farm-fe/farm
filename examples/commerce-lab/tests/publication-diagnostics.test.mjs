import assert from 'node:assert/strict';
import test from 'node:test';
import { createFixtures } from '../packages/domain/index.mjs';

test('publication diagnostics leave merchant validation and immutable business results intact', async () => {
  const module = await import('../packages/domain/publication-diagnostics.mjs').catch(error => {
    if (error.code === 'ERR_MODULE_NOT_FOUND') return {};
    throw error;
  });
  assert.equal(typeof module.publishWithDiagnostics, 'function', 'diagnostic publication wrapper is missing');
  const state = createFixtures();
  const before = JSON.stringify(state);
  const product = state.products.find(row => row.tenantId === 'north' && row.status === 'published');
  const input = { name: 'Diagnostic launch', discount: 10, productIds: [product.id] };
  const config = { version: 'channels-2026-10', channels: ['Store'] };
  assert.equal(module.publishWithDiagnostics(state, 'north', input, 'Store', config).campaigns.at(-1).name, input.name);
  assert.throws(() => module.publishWithDiagnostics(state, 'south', input, 'Store', config), /product/i);
  assert.throws(() => module.publishWithDiagnostics(state, 'north', { ...input, discount: 81 }, 'Store', config), /discount/i);
  assert.equal(JSON.stringify(state), before);
});
