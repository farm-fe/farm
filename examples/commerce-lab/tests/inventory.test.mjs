import assert from 'node:assert/strict';
import test from 'node:test';
import { createFixtures } from '../packages/domain/index.mjs';
import { receiveStock, inventorySummary, parseReceiptUnits, receiptPreview } from '../packages/domain/inventory.mjs';

test('receipt preview shows the same safe stock change as confirmation', () => {
  assert.deepEqual(receiptPreview(18, '37'), { before: 18, received: 37, after: 55 });
  assert.deepEqual(receiptPreview(0, '0009'), { before: 0, received: 9, after: 9 });
  for (const text of ['', '-1', '0', '1.5', '1e2']) assert.throws(() => receiptPreview(18, text), /units/i);
  assert.throws(() => receiptPreview(Number.MAX_SAFE_INTEGER, '1'), /inventory/i);
  for (const stock of [-1, 0.5, Infinity]) assert.throws(() => receiptPreview(stock, '1'), /inventory/i);
  const state = createFixtures();
  const product = state.products[0];
  assert.equal(receiveStock(state, 'north', product.id, 37).products[0].stock, receiptPreview(product.stock, '37').after);
});
test('stock receipts are positive safe integers and merchant scoped', () => {
  const state = createFixtures();
  const product = state.products[0];
  const next = receiveStock(state, 'north', product.id, 37);
  assert.equal(next.products[0].stock, product.stock + 37);
  assert.equal(state.products[0].stock, product.stock);
  for (const units of [0, -1, 1.5, Infinity]) assert.throws(() => receiveStock(state, 'north', product.id, units), /units/i);
  assert.throws(() => receiveStock(state, 'south', product.id, 1), /product/i);
  assert.equal(parseReceiptUnits('37'), 37);
  for (const input of ['', '-3', '1.5', '1e2', '9007199254740992']) assert.throws(() => parseReceiptUnits(input), /units/i);
  const total = inventorySummary(state, 'north');
  const owned = state.products.filter(product => product.tenantId === 'north');
  assert.equal(total.totalUnits, owned.reduce((sum, product) => sum + product.stock, 0));
  assert.equal(total.stockValue, owned.reduce((sum, product) => sum + product.stock * product.price, 0));
});
