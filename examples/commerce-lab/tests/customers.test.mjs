import assert from 'node:assert/strict';
import test from 'node:test';
import { customerProfiles, customerOrders } from '../packages/domain/customers.mjs';
import { createFixtures } from '../packages/domain/index.mjs';

test('fixtures include repeat customers with varied order states and channels', () => {
  const state = createFixtures();
  const orders = state.orders.filter(order => order.tenantId === 'north' && order.customer === 'Customer 1');
  assert.ok(orders.length > 1);
  assert.ok(new Set(orders.map(order => order.status)).size > 1);
  assert.ok(new Set(orders.map(order => order.channel)).size > 1);
});

test('customer profiles keep refunds, revenue and merchant ownership distinct', () => {
  const state = { tenants: [{ id: 'north' }, { id: 'south' }], orders: [
    { id: 'n1', tenantId: 'north', customer: 'Alex', status: 'paid', total: 1000, day: 2, channel: 'Store' },
    { id: 'n2', tenantId: 'north', customer: 'Alex', status: 'completed', total: 2000, day: 4, channel: 'Social' },
    { id: 'n3', tenantId: 'north', customer: 'Alex', status: 'refunded', total: 900, day: 6, channel: 'Store' },
    { id: 's1', tenantId: 'south', customer: 'Alex', status: 'paid', total: 9999, day: 8, channel: 'Marketplace' },
  ] };
  const before = JSON.stringify(state);
  const rows = customerProfiles(state, 'north', ' aLeX ');
  assert.equal(rows.length, 1);
  assert.equal(rows[0].revenue, 3000);
  assert.equal(rows[0].orders, 3);
  assert.equal(rows[0].refundedOrders, 1);
  assert.equal(rows[0].refundedAmount, 900);
  assert.equal(rows[0].lastDay, 6);
  assert.deepEqual(rows[0].channels, ['Social', 'Store']);
  assert.deepEqual(customerOrders(state, 'north', 'Alex').map(order => order.id), ['n3', 'n2', 'n1']);
  assert.equal(customerProfiles(state, 'north', 'absent').length, 0);
  assert.throws(() => customerProfiles(state, 'foreign'), /merchant/i);
  assert.equal(JSON.stringify(state), before);
});
