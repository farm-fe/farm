import assert from 'node:assert/strict';
import test from 'node:test';
import { modularUi, modularUiJsxPrelude } from '../adapters/modular-ui.mjs';
import { extensionSources } from '../benchmark/extension-generator.mjs';

test('JS reference preserves JSX aliases through shared lowering and emits actual Arco component styles', async () => {
  const param = { resolvedPath: '/commerce/ProductBulkActions.tsx', moduleType: 'tsx',
    content: "import { Button as Action, Tooltip as Tip } from '@arco-design/web-react'; export const Controls = () => <Tip><Action>Publish</Action></Tip>;" };
  const lowered = await modularUiJsxPrelude().transform.executor(param);
  const result = await modularUi('js').transform.executor({ ...param, content: lowered.content });
  for (const component of ['Button', 'Tooltip']) {
    assert.ok(result.content.includes(`@arco-design/web-react/es/${component}"`));
    assert.ok(result.content.includes(`@arco-design/web-react/es/${component}/style/index.css`));
  }
  assert.ok(!result.content.includes('import { Button as Action'));
  assert.ok(!result.content.includes('_jsx(Action'));
  assert.ok(!result.content.includes('_jsx(Tip'));
});

test('catalog business JSX uses the existing Babel transform rather than only native React', async () => {
  const plugin = modularUiJsxPrelude({ catalog: true });
  const filename = 'packages/extensions/catalog-256/category-001/VariantTable.tsx';
  assert.ok(plugin.transform.filters.resolvedPaths.some(pattern => new RegExp(pattern).test(filename)));
  const result = await plugin.transform.executor({ resolvedPath: filename, moduleType: 'tsx', content: extensionSources(1, { rich: true })['category-001/VariantTable.tsx'] });
  assert.ok(result.content.includes('react/jsx-runtime'));
  assert.ok(result.content.includes('Variant inventory'));
  assert.ok(!result.content.includes('<table>'));
  assert.ok(result.sourceMap);
});

test('mixed-style business catalog JSX is included in the JS transform workload', async () => {
  const plugin = modularUiJsxPrelude({ catalog: true });
  const filename = 'packages/extensions/business-1024/category-001/PricingPreview.tsx';
  assert.ok(plugin.transform.filters.resolvedPaths.some(pattern => new RegExp(pattern).test(filename)));
  const result = await plugin.transform.executor({ resolvedPath: filename, moduleType: 'tsx', content: extensionSources(1, { business: true })['category-001/PricingPreview.tsx'] });
  assert.ok(result.content.includes('react/jsx-runtime'));
  assert.ok(result.content.includes('Delivery destination'));
  assert.ok(!result.content.includes('<aside'));
});
