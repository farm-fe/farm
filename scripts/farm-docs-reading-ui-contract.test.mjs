import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { test } from 'node:test';
// These are portable source contracts; computed styles and touch remain browser acceptance.
const root = process.env.FARM_DOCS_READING_ROOT || fileURLToPath(new URL('..', import.meta.url));
const css = fs.readFileSync(path.join(root, 'packages/docs-theme-default/src/style.css'), 'utf8');
const card = fs.readFileSync(path.join(root, 'website/src/components/MagicUi/card.tsx'), 'utf8');
const start = css.indexOf('/* SVG ink and the first title line');
const reading = start >= 0 ? css.slice(start) : '';
test('sidebar marker has one grid-aligned chevron without fixed vertical offsets', () => {
 assert.match(reading, /grid-template-columns: 14px minmax\(0, 1fr\)/);
 assert.match(reading, /summary::marker \{ content: ''/);
 assert.match(reading, /summary::before \{ content: none; display: none/);
 assert.match(reading, /farm-docs-sidebar-chevron \{[^}]*height: 1\.5em/);
 assert.match(reading, /details\[open\] > summary > \.farm-docs-sidebar-chevron/);
});
test('tabs scroll within their own strip and expose selected state styling', () => {
 assert.match(reading, /flex-wrap: nowrap/);
 assert.match(reading, /overflow-x: auto; overflow-y: hidden/);
 assert.match(reading, /> \[role="tab"\]\[aria-selected="true"\]/);
 assert.match(reading, /min-width: 0; max-width: 100%/);
});
test('icon copy has a 44px target and touch visibility without changing token generation', () => {
 assert.match(reading, /width: 44px; height: 44px; min-width: 44px; min-height: 44px/);
 assert.match(reading, /farm-docs-code-copy svg \{ width: 19px; height: 19px/);
 assert.match(reading, /@media \(hover: none\), \(pointer: coarse\), \(max-width: 996px\)/);
 assert.doesNotMatch(reading, /farm-code-token/);
});
test('block code uses the verified source site rhythm and local overflow', () => {
 assert.match(reading, /font-size: \.95rem; line-height: 1\.45/);
 assert.match(reading, /padding: 1rem; padding-inline-end: 60px/);
 assert.match(reading, /background: transparent; overflow-x: auto/);
 assert.match(reading, /--farm-docs-code-bg: #fbfbfb/);
});
test('inline code has an explicit contrasting surface in ordinary content and notices', () => {
 assert.match(reading, /--farm-docs-inline-bg: #f1edf4/);
 assert.match(reading, /--farm-docs-admonition-code-bg: #ffffff/);
 assert.match(reading, /--farm-docs-inline-bg: #302936/);
 assert.match(reading, /--farm-docs-admonition-code-bg: #19151f/);
 assert.match(reading, /article \.farm-admonition :not\(pre\) > code/);
 assert.match(reading, /background: var\(--farm-docs-admonition-code-bg/);
});
test('homepage retains feature images and content while omitting decorative corner icons', () => {
 for (const image of ['Box', 'Compatibility', 'FeaturePng', 'Rocket']) assert.match(card, new RegExp('src=\\{' + image + '\\}'));
 for (const name of ['fast', 'incremental', 'bundling', 'pluggable', 'compatibility']) {
  assert(card.includes('features.' + name + '.title')); assert(card.includes('features.' + name + '.description'));
 }
 assert.match(card, /<AnimatedBeamPig \/>/);
 assert.doesNotMatch(card, /icon=\{item.icon\}|icon: <Icon/);
});

test('highlighted lines preserve the same horizontal origin as ordinary lines', () => {
 assert.match(reading, /pre \.code-block-highlight-line \{ margin: 0; padding: 0; border: 0/);
});

test('native highlighted lines use a non-layout accent rather than a shifting border', () => {
 const nativeCss = fs.readFileSync(path.join(root, 'rust-plugins/syntax-highlight/style.css'), 'utf8');
 const rule = nativeCss.match(/\.farm-code-line\[data-highlighted="true"\] \{([^}]+)\}/)?.[1];
 assert(rule);
 assert.match(rule, /border: 0/);
 assert.match(rule, /box-shadow: inset/);
 assert.doesNotMatch(rule, /padding|margin|border-inline-start/);
});
