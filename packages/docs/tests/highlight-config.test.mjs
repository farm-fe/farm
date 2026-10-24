import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import { test } from 'node:test';
const require = createRequire(import.meta.url);
const ts = require('typescript');
const source = fs.readFileSync(new URL('../src/highlight-config.ts', import.meta.url), 'utf8');
const code = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ES2022 } }).outputText;
const { resolveDocsCodeTheme: resolve } = await import('data:text/javascript;base64,' + Buffer.from(code).toString('base64'));
const base = { title: 'Docs', url: 'https://example.com' };

test('website brand palette reaches every native token category without legacy color overrides', () => {
  const site = JSON.parse(fs.readFileSync(new URL('../../../website/farm-docs.site.json', import.meta.url), 'utf8'));
  const result = resolve(site, { highlight: { light: 'vscode-light', dark: 'vscode-dark' } });
  assert.deepEqual(result.highlight.tokens, site.code.tokens);
  assert.equal(result.highlight.light, 'farm-light');
  assert.equal(result.highlight.dark, 'farm-dark');
  const luminance = hex => {
    const channels = [1, 3, 5].map(i => Number.parseInt(hex.slice(i, i + 2), 16) / 255);
    return channels.map(c => c <= .04045 ? c / 12.92 : ((c + .055) / 1.055) ** 2.4)
      .reduce((sum, c, i) => sum + c * [.2126, .7152, .0722][i], 0);
  };
  for (const [mode, background] of [['light', '#faf7fb'], ['dark', '#211d25']]) {
    assert.equal(Object.keys(result.highlight.tokens[mode]).length, 16);
    for (const [category, foreground] of Object.entries(result.highlight.tokens[mode])) {
      const values = [luminance(background), luminance(foreground)].sort((a, b) => a - b);
      assert((values[1] + .05) / (values[0] + .05) >= 4.5, `${mode} ${category} contrast`);
    }
  }
});
test('legacy palette is resolved into the site before native prepare, not snippets only', () => {
  const out = resolve(base, { highlight: { light: 'inspired-github', dark: 'vscode-dark' } });
  assert.deepEqual(out.site.code, { light: 'inspired-github', dark: 'vscode-dark' });
  assert.equal(out.highlight.light, out.site.code.light);
  assert.equal(out.highlight.dark, out.site.code.dark);
  assert.equal(base.code, undefined);
});
test('explicit site fields override legacy while missing fields still fall back', () => {
  const out = resolve({ ...base, code: { light: 'farm-light' } }, { highlight: { light: 'vscode-light', dark: 'farm-dark' } });
  assert.deepEqual(out.site.code, { light: 'farm-light', dark: 'farm-dark' });
  assert.equal(out.highlight.light, 'farm-light');
});
test('custom token maps merge by mode and category without mutating caller options', () => {
  const old = { highlight: { tokens: { light: { keyword: '#123456', string: '#654321' } } } };
  const current = { ...base, code: { tokens: { light: { keyword: '#aabbcc' }, dark: { type: '#ddeeff' } } } };
  const out = resolve(current, old);
  assert.deepEqual(out.site.code.tokens, { light: { keyword: '#aabbcc', string: '#654321' }, dark: { type: '#ddeeff' } });
  assert.deepEqual(old.highlight.tokens.light, { keyword: '#123456', string: '#654321' });
  assert.deepEqual(resolve(out.site, old).site, out.site, 'repeated composition is idempotent');
});
test('build representation has one authority and does not become a palette field', () => {
  const out = resolve({ ...base, build: { highlightRepresentation: 'static-html' } }, { highlight: { representation: 'jsx' } });
  assert.equal(out.highlight.representation, 'static-html');
  assert.deepEqual(out.site.code, {});
});
test('invalid palette objects are rejected, and unknown native fields are not silently removed', () => {
  for (const value of [null, [], 'farm-light']) assert.throws(() => resolve({ ...base, code: value }), /must be an object/);
  assert.throws(() => resolve({ ...base, code: { tokens: { dark: [] } } }), /must be an object/);
  assert.equal(resolve({ ...base, code: { typo: true } }).site.code.typo, true);
});
