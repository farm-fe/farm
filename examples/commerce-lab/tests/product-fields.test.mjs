import assert from 'node:assert/strict';
import test from 'node:test';
import { productFields } from '../packages/domain/product-fields.mjs';

test('product form converts decimal currency without floating point rounding', () => {
  assert.deepEqual(productFields('Weekend tote', '19.99', '25'), { name: 'Weekend tote', price: 1999, stock: 25 });
  assert.equal(productFields('Free sample', '0', '0').price, 0);
  assert.equal(productFields('Item', '1.2', '1').price, 120);
});
test('invalid price, fractional inventory and oversized amounts are rejected', () => {
  for (const price of ['', '-1', '1.001', 'NaN', '1e3', '9007199254740992']) {
    assert.throws(() => productFields('Item', price, '1'), /price/i);
  }
  for (const stock of ['', '-1', '1.5', 'Infinity', '9007199254740992']) {
    assert.throws(() => productFields('Item', '1', stock), /stock/i);
  }
  assert.throws(() => productFields('  ', '1', '1'), /name/i);
});
