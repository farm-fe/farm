import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
const read = file => fs.readFileSync(new URL('../' + file, import.meta.url), 'utf8');
test('website points to current usage and distinguishes historical records', () => {
  const source = read('website/README.md');
  assert.ok(source.includes('(docs/farm-docs/usage.md)'));
  assert.ok(fs.existsSync(new URL('../website/docs/farm-docs/usage.md', import.meta.url)));
  assert.ok(source.includes('Historical migration'));
  assert.ok(!source.includes('../docs/farm-docs/README.md'));
});
test('Markdown actions are explicitly enabled by custom themes', () => {
  for (const file of ['packages/docs/README.md', 'packages/docs-theme-default/README.md']) assert.match(read(file), /custom theme.*explicitly\s+render/is);
  const source = read('packages/docs-theme-default/src/index.tsx');
  assert.doesNotMatch(source.slice(source.indexOf('export function DocPage('), source.indexOf('export function BlogPage(')), /<DocumentActions(?:\s|\/|>)/);
});
test('retired editorial homepage CSS is absent while Footer keeps its styles', () => {
  const css = read('website/src/css/farm-site.css');
  assert.doesNotMatch(css, /farm-editorial-(?!footer)[a-z-]+/);
  assert.match(css, /\.farm-editorial-footer-top/);
  assert.match(css, /\.farm-editorial-footer nav/);
  assert.match(read('website/src/theme/Footer/index.tsx'), /farm-editorial-footer/);
  assert.match(css, /farm-online-home/);
});
