import assert from 'node:assert/strict';
import test from 'node:test';
import { createFixtures } from '../packages/domain/index.mjs';
import { createMockApi } from '../packages/domain/api.mjs';

test('API returns tenant-scoped data and rejects a forged tenant', async () => {
  const state = createFixtures();
  const api = createMockApi({ state: () => state, tenantId: 'north', role: 'admin', delay: 1 });
  const response = await api.request('products');
  assert.equal(response.length, 60);
  assert.ok(response.every(p => p.tenantId === 'north'));
  await assert.rejects(api.request('products', { tenantId: 'south' }), error => error.status === 403);
});

test('API exposes deterministic timeout and permission states', async () => {
  const options = { state: () => createFixtures(), tenantId: 'north', role: 'viewer', delay: 1 };
  await assert.rejects(createMockApi(options).request('campaigns'), error => error.status === 403);
  await assert.rejects(createMockApi({ ...options, mode: 'timeout' }).request('products'), error => error.status === 408);
});

test('cancelled request cannot return stale merchant data', async () => {
  const controller = new AbortController();
  const api = createMockApi({ state: createFixtures, tenantId: 'north', role: 'admin', delay: 100 });
  const pending = api.request('products', { signal: controller.signal });
  controller.abort();
  await assert.rejects(pending, error => error.name === 'AbortError');
});
