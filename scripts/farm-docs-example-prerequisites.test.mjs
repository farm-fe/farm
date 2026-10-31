import assert from 'node:assert/strict';
import fs from 'node:fs';
import { test } from 'node:test';
import { docsExamplePrerequisites } from './build.mjs';

test('Docs examples include their transitive native plugins and ordered public packages', () => {
  for (const name of ['docs-features', 'docs-minimal']) {
    const pkg = JSON.parse(fs.readFileSync(new URL(`../examples/${name}/package.json`, import.meta.url)));
    const result = docsExamplePrerequisites(pkg);
    assert.deepEqual(result.public, ['docs', 'docs-theme-default']);
    assert.deepEqual([...result.native].sort(), ['docs', 'docs-seo', 'docs-snapshot', 'docs-snippets', 'react', 'ssr-css', 'syntax-highlight']);
    for (const plugin of result.native) {
      const manifest = JSON.parse(fs.readFileSync(new URL(`../rust-plugins/${plugin}/package.json`, import.meta.url)));
      assert.ok(manifest.scripts.build);
    }
  }
});

test('ordinary examples do not acquire Docs prerequisites', () => {
  assert.deepEqual(docsExamplePrerequisites({ dependencies: { react: '18.2.0' } }), { native: [], public: [] });
});
