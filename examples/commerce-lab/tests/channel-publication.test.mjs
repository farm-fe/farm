import assert from 'node:assert/strict';
import test from 'node:test';
import { createFixtures } from '../packages/domain/index.mjs';

test('channel publication preserves merchant validation and stamps the selected public rule', async () => {
  const module = await import('../packages/domain/channel-publication.mjs').catch(error => {
    if (error.code === 'ERR_MODULE_NOT_FOUND') return {};
    throw error;
  });
  assert.equal(typeof module.publishToChannel, 'function', 'Channel-aware publication is not implemented');
  const state = createFixtures();
  const before = JSON.stringify(state);
  const product = state.products.find(row => row.tenantId === 'north' && row.status === 'published');
  const input = { name: 'Store launch', discount: 10, productIds: [product.id] };
  const config = { version: 'channels-2026-10', channels: ['Store', 'Social'] };
  const next = module.publishToChannel(state, 'north', input, 'Store', config);
  assert.equal(next.campaigns.at(-1).channel, 'Store');
  assert.equal(next.campaigns.at(-1).channelRuleVersion, 'channels-2026-10');
  assert.equal(JSON.stringify(state), before);
  assert.throws(() => module.publishToChannel(state, 'north', input, 'Marketplace', config), /channel/i);
  assert.throws(() => module.publishToChannel(state, 'south', input, 'Store', config), /product/i);
  assert.throws(() => module.publishToChannel(state, 'unknown', input, 'Store', config), /merchant|tenant/i);
  assert.throws(() => module.publishToChannel(state, 'north', { ...input, discount: 81 }, 'Store', config), /discount/i);
  assert.throws(() => module.publishToChannel(state, 'north', input, 'Store', { ...config, version: '' }), /configuration/i);
});
