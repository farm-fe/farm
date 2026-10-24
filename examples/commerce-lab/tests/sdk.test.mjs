import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import test from 'node:test';
import ts from 'typescript';

const source = await fs.readFile(new URL('../packages/sdk/src/index.ts', import.meta.url), 'utf8');
const { outputText } = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.ESNext } });
const { quoteCampaign } = await import(`data:text/javascript;base64,${Buffer.from(outputText).toString('base64')}`);
const products = [{ id: 'north-p1', tenantId: 'north', price: 1999, stock: 3 }];

test('SDK campaign quote uses integer cents and preserves its input', () => {
  const before = structuredClone(products);
  assert.deepEqual(quoteCampaign('north', products, 15), {
    tenantId: 'north', discount: 15,
    items: [{ productId: 'north-p1', originalPrice: 1999, salePrice: 1699, availableStock: 3 }],
  });
  assert.deepEqual(products, before);
});
test('SDK rejects foreign products and invalid financial inputs', () => {
  assert.throws(() => quoteCampaign('south', products, 15), /merchant/);
  for (const discount of [-1, 101, NaN, 10.5]) assert.throws(() => quoteCampaign('north', products, discount), /discount/);
  assert.throws(() => quoteCampaign('north', [{ ...products[0], price: 2.5 }], 15), /price/);
});
