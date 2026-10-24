import assert from 'node:assert/strict';
import test from 'node:test';
import { createFixtures } from '../packages/domain/index.mjs';
import { filterProducts } from '../packages/domain/product-query.mjs';
test('catalog filters intersect merchant, search, category and publication status', () => {
  const state = createFixtures(); const before = JSON.stringify(state);
  const expected = state.products.filter(product => product.tenantId === 'north' && product.category === 'Outdoor' && product.status === 'draft');
  assert.deepEqual(filterProducts(state, 'north', { search: ' ceramic ', category: 'Outdoor', status: 'draft' }), expected);
  assert.deepEqual(filterProducts(state, 'north', { search: 'SOUTH-' }), []);
  assert.deepEqual(filterProducts(state, 'north', { category: 'no-such-category' }), []);
  assert.throws(() => filterProducts(state, 'missing'), /merchant/i);
  assert.throws(() => filterProducts(state, 'north', { status: 'invalid' }), /status/i);
  assert.equal(JSON.stringify(state), before);
});
