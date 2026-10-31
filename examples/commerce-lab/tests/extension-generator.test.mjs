import assert from 'node:assert/strict';
import test from 'node:test';
import { extensionSources } from '../benchmark/extension-generator.mjs';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import ts from 'typescript';

test('extension source generator is deterministic and imports every interactive category', () => {
  const sources = extensionSources(2);
  assert.deepEqual(sources, extensionSources(2));
  assert.equal(Object.keys(sources).length, 9);
  for (const id of ['category-001', 'category-002']) {
    assert.ok(sources['index.ts'].includes(`./${id}/Form`));
    assert.ok(sources[`${id}/Form.tsx`].includes('./Guide.mdx'));
    assert.ok(sources[`${id}/Form.tsx`].includes('./icon.svg'));
    assert.ok(sources[`${id}/Form.tsx`].includes('./style.scss'));
    assert.ok(sources[`${id}/Form.tsx`].includes('onSubmit'));
    assert.ok(sources[`${id}/Form.tsx`].includes('useState'));
  }
  for (const count of [0, -1, 1.5, 4097]) assert.throws(() => extensionSources(count), /count/i);
});

test('large catalog keeps every real JSX component reachable without new dependencies', () => {
  const files = extensionSources(256, { rich: true });
  assert.equal(Object.keys(files).length, 2561);
  for (const id of ['category-001', 'category-256']) {
    assert.ok(files['index.ts'].includes(`./${id}/Form`));
    for (const name of ['AttributeFields', 'ValidationSummary', 'VariantTable', 'PricingPreview']) {
      assert.ok(files[`${id}/Form.tsx`].includes(`./${name}`));
      assert.ok(files[`${id}/${name}.tsx`]);
    }
    assert.ok(files[`${id}/VariantTable.tsx`].includes('./AvailabilityBadge'));
    assert.ok(files[`${id}/AvailabilityBadge.jsx`]);
    assert.ok(files[`${id}/Form.tsx`].includes('validateAttributes'));
    assert.ok(files[`${id}/PricingPreview.tsx`].includes('quoteVariant'));
  }
});

test('generated catalog validates actual attributes, prices and inventory and computes cent-accurate quotes', async () => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'commerce-catalog-model-'));
  await fs.writeFile(path.join(directory, 'package.json'), '{"type":"module"}');
  const model = path.join(directory, 'model.mjs');
  const source = extensionSources(1, { rich: true })['category-001/model.ts'];
  const { outputText } = ts.transpileModule(source, {
    compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
  });
  await fs.writeFile(model, outputText);
  const { validateAttributes, initialVariants, quoteVariant } = await import(pathToFileURL(model).href);
  assert.deepEqual(validateAttributes({}, initialVariants), ['Dimensions is required']);
  assert.deepEqual(validateAttributes({ Dimensions: '40 x 60 cm' }, initialVariants), []);
  assert.match(validateAttributes({ Dimensions: 'x'.repeat(201) })[0], /exceeds 200/);
  const invalid = [{ ...initialVariants[0], price: -1, stock: 1.5 }];
  assert.equal(validateAttributes({ Dimensions: '40 x 60 cm' }, invalid).length, 2);
  assert.equal(quoteVariant(1999, 2, 10), 3598);
  for (const args of [[0, 1, 0], [1999, 0, 0], [1999, 1, 51], [1999, 1, NaN], [1e15, 10, 0]]) assert.throws(() => quoteVariant(...args), /quote/);
});
