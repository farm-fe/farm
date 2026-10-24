import assert from 'node:assert/strict';
import test from 'node:test';
import { extensionSources } from '../benchmark/extension-generator.mjs';

test('business catalog connects rules, tier prices, shipping and mixed styling to the editor', () => {
  const files = extensionSources(2, { rich: true, business: true });
  for (const id of ['category-001', 'category-002']) {
    assert.ok(files[`${id}/rules.yaml`]);
    assert.ok(files[`${id}/tiers.csv`]);
    assert.ok(files[`${id}/shipping.tsv`]);
    const model = files[`${id}/business.ts`];
    for (const input of ['./rules.yaml', './tiers.csv', './shipping.tsv']) assert.ok(model.includes(input));
    assert.ok(files[`${id}/PricingPreview.tsx`].includes('./business'));
    assert.ok(files[`${id}/Form.tsx`].includes('./business'));
    assert.ok(files[`${id}/Form.tsx`].includes('md:grid-cols-2'));
    assert.ok(files[`${id}/style.scss`].includes('.availability-low'));
  }
});

test('business quote enforces stock and rule limits, applies tiers and destination shipping', async () => {
  const { businessQuote } = await import('../benchmark/business-quote.mjs');
  const rules = { maxDiscount: 20, reserveStock: 2 };
  const tiers = [{ minQuantity: '1', discount: '0' }, { minQuantity: '5', discount: '10' }];
  const shipping = [{ destination: 'domestic', cents: '500', freeAbove: '10000' }, { destination: 'export', cents: '1500', freeAbove: '20000' }];
  const quote = (patch = {}) => businessQuote({ price: 2000, stock: 12, quantity: 5, discount: 5, destination: 'domestic', rules, tiers, shipping, ...patch });
  assert.deepEqual(quote(), { subtotal: 10000, discount: 15, shipping: 500, total: 9000 });
  assert.equal(quote({ destination: 'export' }).total, 10000);
  assert.equal(quote({ quantity: 10, discount: 0 }).shipping, 0);
  assert.throws(() => quote({ quantity: 11 }), /stock/);
  assert.throws(() => quote({ discount: 21 }), /discount/);
  assert.throws(() => quote({ destination: 'unknown' }), /destination/);
  assert.throws(() => quote({ price: NaN }), /price/);
});
