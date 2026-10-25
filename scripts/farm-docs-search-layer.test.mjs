import assert from 'node:assert/strict';
import fs from 'node:fs';
import { test } from 'node:test';

const css = fs.readFileSync(new URL('../packages/docs-theme-default/src/style.css', import.meta.url), 'utf8')
  .replace(/\/\*[\s\S]*?\*\//g, '');
// Static integration guards; browser geometry and focus still require acceptance.
function rule(selector) {
  const matches = [...css.matchAll(/([^{}]+)\{([^{}]*)\}/g)]
    .filter(match => match[1].trim() === selector);
  assert.ok(matches.length, `Missing rule: ${selector}`);
  return matches.map(match => match[2]).join(';');
}

test('body portal search overlay sits above website navigation', () => {
  const overlay = rule('body > .DocSearch-Container');
  const layer = Number(overlay.match(/z-index:\s*(\d+)/)?.[1]);
  const navigationLayers = [...css.matchAll(/([^{}]+)\{([^{}]*z-index:\s*(\d+)[^{}]*)\}/g)]
    .filter(match => /farm-docs-(?:navbar|navigation-menu|mobile-dialog)/.test(match[1]))
    .map(match => Number(match[3]));
  assert.ok(navigationLayers.length);
  assert.ok(navigationLayers.every(value => value < layer));
  assert.match(overlay, /inset:\s*0\s*;/);
  assert.match(overlay, /(?:^|;)\s*width:\s*100%\s*;/);
  assert.doesNotMatch(overlay, /100vw/);
});

test('document reserves its scrollbar gutter when search locks body scrolling', () => {
  assert.match(rule(':root'), /scrollbar-gutter:\s*stable\s*;/);
});

test('website search continues using the vendor portal and native modal owns fallback focus', () => {
  const website = fs.readFileSync(new URL('../website/src/docs/search.tsx', import.meta.url), 'utf8');
  const theme = fs.readFileSync(new URL('../packages/docs-theme-default/src/search.tsx', import.meta.url), 'utf8');
  assert.match(website, /import\s*\{\s*DocSearch\s*\}\s*from\s*'@docsearch\/react'/);
  assert.match(website, /<DocSearch\s/);
  assert.match(theme, /dialog\.current\.showModal\(\)/);
  assert.match(theme, /onCancel=\{/);
});
