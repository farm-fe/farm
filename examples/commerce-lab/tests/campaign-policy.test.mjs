import assert from 'node:assert/strict';
import test from 'node:test';
import { createFixtures, createCampaign } from '../packages/domain/index.mjs';
import { campaignPolicy, updateCampaignPolicy } from '../packages/domain/campaign-policy.mjs';
test('merchant discount policy is immutable, bounded and enforced at publication', () => {
  const state = createFixtures(); const before = JSON.stringify(state);
  const next = updateCampaignPolicy(state, 'north', 20);
  assert.equal(campaignPolicy(next, 'north').maxDiscount, 20);
  assert.equal(campaignPolicy(next, 'south').maxDiscount, 80);
  assert.equal(JSON.stringify(state), before);
  for (const value of [0, -1, 81, 1.5, Infinity]) assert.throws(() => updateCampaignPolicy(state, 'north', value), /discount/i);
  assert.throws(() => updateCampaignPolicy(state, 'foreign', 20), /merchant/i);
  const product = state.products.find(product => product.tenantId === 'north' && product.status === 'published');
  assert.throws(() => createCampaign(next, 'north', { name: 'Over limit', discount: 21, productIds: [product.id] }), /discount/i);
  assert.equal(createCampaign(next, 'north', { name: 'At limit', discount: 20, productIds: [product.id] }).campaigns.at(-1).discount, 20);
});
test('lowering a policy leaves the already published offer unchanged', () => {
  const state = createFixtures();
  const product = state.products.find(product => product.tenantId === 'north' && product.status === 'published');
  const published = createCampaign(state, 'north', { name: 'Historic launch', discount: 60, productIds: [product.id] });
  const limited = updateCampaignPolicy(published, 'north', 20);
  assert.deepEqual(limited.campaigns, published.campaigns);
  assert.equal(limited.campaigns[0].discount, 60);
  assert.throws(() => createCampaign(limited, 'north', { name: 'New launch', discount: 60, productIds: [product.id] }), /discount/i);
});
