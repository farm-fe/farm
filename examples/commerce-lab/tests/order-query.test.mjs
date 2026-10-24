import assert from 'node:assert/strict';
import test from 'node:test';
import { createFixtures } from '../packages/domain/index.mjs';
import { queryOrders, orderSummary } from '../packages/domain/order-query.mjs';
test('order search intersects merchant, status, channel and inclusive date range', () => {
  const state = createFixtures(); const before = JSON.stringify(state);
  const expected = state.orders.filter(order => order.tenantId === 'north' && order.status === 'shipped' && order.channel === 'Social' && order.day >= 2 && order.day <= 4 && order.productId === 'north-p1');
  assert.deepEqual(queryOrders(state, 'north', { search: ' north-1001 ', status: 'shipped', channel: 'Social', fromDay: 2, toDay: 4 }), expected);
  assert.deepEqual(queryOrders(state, 'north', { search: 'north-o0' }).map(order => order.id), ['north-o0']);
  assert.ok(queryOrders(state, 'north', { search: 'customer 1' }).every(order => order.customer.toLowerCase().includes('customer 1')));
  assert.deepEqual(queryOrders(state, 'north', { search: 'no-result' }), []);
  assert.throws(() => queryOrders(state, 'unknown'), /merchant/i);
  for (const options of [{ status: 'invalid' }, { channel: 'invalid' }, { fromDay: 31 }, { fromDay: 4, toDay: 2 }]) assert.throws(() => queryOrders(state, 'north', options), /filter/i);
  assert.equal(JSON.stringify(state), before);
});
test('filtered order summary separates net charges and refunds', () => {
  assert.deepEqual(orderSummary([{ total: 1000, quantity: 2, status: 'paid' }, { total: 500, quantity: 1, status: 'refunded' }]), { orders: 2, netRevenue: 1000, refundedAmount: 500, units: 3 });
  assert.deepEqual(orderSummary([]), { orders: 0, netRevenue: 0, refundedAmount: 0, units: 0 });
});
