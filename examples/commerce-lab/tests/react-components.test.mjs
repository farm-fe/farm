import assert from 'node:assert/strict';
import test from 'node:test';
import { reactComponents } from '../adapters/react-components.mjs';

test('local named JSX exports are discovered and imported into their actual consumer', async () => {
  const plugin = reactComponents('js');
  const result = await plugin.transform.executor({ resolvedPath: `${process.cwd()}/apps/admin/components/ProductImagePreview.tsx`, moduleType: 'tsx',
    content: 'export const Preview = () => <ProductImageMetadata category="Home" name="Canvas"/>;' });
  assert.match(result.content, /import \{ ProductImageMetadata \} from "\.\/ProductImageMetadata\.tsx"/);
});

test('explicit aliases and local bindings take precedence in the JS discovery reference', async () => {
  const plugin = reactComponents('js');
  for (const content of [
    "import { Other as ProductImageMetadata } from './explicit'; export const Preview = () => <ProductImageMetadata/>;",
    'export function Preview({ ProductImageMetadata }) { return <ProductImageMetadata/>; }',
  ]) {
    const result = await plugin.transform.executor({ resolvedPath: `${process.cwd()}/apps/admin/components/ProductImagePreview.tsx`, moduleType: 'tsx', content });
    assert.ok(!result.content.includes('./ProductImageMetadata.tsx'));
  }
});
