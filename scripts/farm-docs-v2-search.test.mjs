import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import { test } from 'node:test';
import ts from 'typescript';

const site = JSON.parse(fs.readFileSync(new URL('../website/farm-docs.site.json', import.meta.url), 'utf8'));
const source = fs.readFileSync(new URL('../website/src/docs/search.tsx', import.meta.url), 'utf8');
const compiled = ts.transpileModule(source, { compilerOptions: {
  module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022,
  jsx: ts.JsxEmit.ReactJSX
} }).outputText;

// Component contract test with vendor/network and React hooks stubbed. Browser
// modal focus/navigation acceptance is deliberately separate from this guard.
function render(locale, mounted = true, config = site) {
  const exports = {};
  vm.runInNewContext(compiled, { exports, require(name) {
    if (name === '@docsearch/react') return { DocSearch: 'DocSearch' };
    if (name === '@docsearch/css') return {};
    if (name === 'react') return {
      createContext: value => ({ value }), useContext: () => locale,
      useEffect() {}, useState: () => [mounted, () => {}]
    };
    if (name === 'react/jsx-runtime') return { jsx: (type, props, key) => ({ type, props, key }) };
    throw new Error(`Unexpected module: ${name}`);
  } });
  return exports.WebsiteSearch({ site: config });
}

test('website switches only to the dedicated V2 search application and index', () => {
  assert.equal(site.search.options.appId, 'X56UDGYU8P');
  assert.equal(site.search.options.indexName, 'farm-v2-docsearch');
  assert.ok(site.search.options.apiKey);
  assert.equal(render('en').props.apiKey, site.search.options.apiKey);
});
test('current locale filters results and remounts vendor state on language changes', () => {
  const en = render('en'), zh = render('zh');
  assert.equal(JSON.stringify(en.props.searchParameters.facetFilters), '["lang:en"]');
  assert.equal(JSON.stringify(zh.props.searchParameters.facetFilters), '["lang:zh"]');
  assert.notEqual(en.key, zh.key);
  assert.equal(render('unknown').key, en.key);
  const theme = fs.readFileSync(new URL('../website/src/docs/theme.tsx', import.meta.url), 'utf8');
  assert.match(theme, /<WebsiteSearchLocale.Provider value=\{props.page.locale\}>/);
});
test('SSR, disabled search and absent credentials never mount the vendor component', () => {
  assert.equal(render('en', false), null);
  assert.equal(render('en', true, { search: { provider: 'none' } }), null);
  assert.equal(render('en', true, { search: { provider: 'algolia' } }), null);
});
