import assert from 'node:assert/strict';
import { test } from 'node:test';
import fs from 'node:fs';
import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
const ts = require(process.env.FARM_DOCS_TEST_TYPESCRIPT || 'typescript');
const source = fs.readFileSync(new URL('../packages/docs-theme-default/src/sidebar-section.ts', import.meta.url), 'utf8');
const compiled = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext } });
const { filterSidebarSection, sidebarSection } = await import('data:text/javascript;base64,' + Buffer.from(compiled.outputText).toString('base64'));
const sections = [{ id: 'guide', prefixes: [] }, { id: 'config', prefixes: ['config'] }, { id: 'plugins', prefixes: ['plugins'] }, { id: 'ecosystem', prefixes: ['plugins/docs'] }];
const doc = id => ({ type: 'doc', id, key: id, label: id, href: '/' + id });
test('most specific configured prefix wins without accidental partial matching', () => {
 assert.equal(sidebarSection('plugins/docs/setup', sections, 'guide'), 'ecosystem');
 assert.equal(sidebarSection('plugins/other', sections, 'guide'), 'plugins');
 assert.equal(sidebarSection('configuration', sections, 'guide'), 'guide');
});
test('section filtering preserves nested order and never mutates the full breadcrumb source', () => {
 const original = [{ type: 'category', label: 'All', items: [doc('plugins/one'), doc('config/two'), { type: 'category', label: 'Docs', items: [doc('plugins/docs/start')] }] }];
 const before = JSON.stringify(original);
 const filtered = filterSidebarSection(original, sections, 'ecosystem', 'guide');
 assert.deepEqual(filtered, [{ type: 'category', label: 'All', items: [{ type: 'category', label: 'Docs', items: [doc('plugins/docs/start')] }] }]);
 assert.equal(JSON.stringify(original), before);
 assert.deepEqual(filterSidebarSection(original, sections, 'missing', 'guide'), []);
});

test('auxiliary links retain order within visible groups without leaking empty groups', () => {
 const link = {type: 'link', href: 'https://example.com', label: 'Reference'};
 const items = [{type:'category', label:'Plugins', items:[doc('plugins/one'), link, doc('plugins/docs/start')]}];
 assert.deepEqual(filterSidebarSection(items, sections, 'plugins', 'guide')[0].items, [doc('plugins/one'), link]);
 assert.deepEqual(filterSidebarSection(items, sections, 'config', 'guide'), []);
});
