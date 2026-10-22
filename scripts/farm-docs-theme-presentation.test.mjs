import { DocsProvider } from '../packages/docs/dist/client.js';
import { CommandTabs } from '../packages/docs/dist/snippets-runtime.js';
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createRequire } from 'node:module';
const require = createRequire(new URL('../packages/docs/package.json', import.meta.url));
const React = require('react');
const { renderToStaticMarkup } = require('react-dom/server');
import { Sidebar, DocPage } from '../packages/docs-theme-default/dist/index.js';

const doc = (id, key = id) => ({ type: 'doc', id, key, label: id, href: `/guide/${id}` });
const section = (label, items) => ({ type: 'category', label, items });
const page = { key: 'active', collection: 'custom', kind: 'doc', headings: [{ depth: 2, id: 'section', title: 'Section' }], title: 'Current document', version: 'v2', locale: 'en' };
const data = { home: '/guide/', sidebars: [{ collection: 'custom', items: [section('Introduction', [doc('first')]), section('Reference', [section('Nested', [doc('unicode/路径', 'active')])]), section('Other', [doc('last')])] }] };

test('theme preserves configured sidebar order, opens active ancestors and accepts initial expansion', () => {
  const markup = renderToStaticMarkup(React.createElement(Sidebar, { page, data, expandedCategories: ['Introduction'] }));
  assert.match(markup, /<details open=""><summary>Introduction/);
  assert.match(markup, /<details open=""><summary>Reference/);
  assert.match(markup, /<details open=""><summary>Nested/);
  assert.match(markup, /<details><summary>Other/);
  assert.match(markup, /<a(?=[^>]*href="\/guide\/unicode\/路径")(?=[^>]*aria-current="page")[^>]*>/);
  assert(markup.indexOf('Introduction') < markup.indexOf('Reference'));
  assert(markup.indexOf('Reference') < markup.indexOf('Other'));
  assert.equal((markup.match(/<a /g) ?? []).length, 3);
});

test('theme uses host-provided labels without changing fallback content or page navigation', () => {
  const current = { ...page, fallbackSource: { locale: 'zh', routePath: '/zh/source' }, previous: { href: '/before', label: 'Before' }, next: { href: '/after', label: 'After' } };
  const markup = renderToStaticMarkup(React.createElement(DocPage, { page: current, data, homeLabel: '首页', versionLabel: '2.x (Latest)' }, React.createElement('h1', null, current.title)));
  assert.match(markup, /aria-label="Breadcrumbs"/);
  assert.match(markup, /<span>Reference<\/span>/);
  assert.match(markup, /<span>Nested<\/span>/);
  assert.doesNotMatch(markup, /<span>Other<\/span>/);
  assert.match(markup, /<details class="farm-docs-mobile-toc"><summary>On this page<\/summary>/);
  assert.match(markup, /href="#section"/);
  assert.match(markup, /href="\/guide\/">首页/);
  assert.match(markup, /Version: 2.x \(Latest\)/);
  assert.match(markup, /href="\/zh\/source"/);
  assert.match(markup, /aria-label="Previous: Before"/);
  assert.match(markup, /aria-label="Next: After"/);
  assert.match(markup, /<h1>Current document<\/h1>/);
});

test('generic website pages do not acquire documentation metadata', () => {
  const markup = renderToStaticMarkup(React.createElement(DocPage, { page: { ...page, kind: 'page' }, data }, React.createElement('p', null, 'Home content')));
  assert.doesNotMatch(markup, /Breadcrumbs|Version:/);
  assert.match(markup, /Home content/);
});

test('decorated command labels preserve compiled commands and every SSR panel', () => {
  const panels = ['npm', 'yarn'].map(tool => React.createElement('pre', { key: tool, 'data-command-tool': tool, 'data-code': `${tool} create farm@latest` }, `${tool} create farm@latest`));
  const labels = { npm: React.createElement('span', null, React.createElement('img', { src: '/npm.svg', alt: '', 'aria-hidden': true }), 'npm') };
  const markup = renderToStaticMarkup(React.createElement(DocsProvider, { site: { basePath: '/', theme: { colorMode: 'light' } }, page }, React.createElement(CommandTabs, { labels }, panels)));
  assert.match(markup, /src="\/npm.svg"/);
  assert.match(markup, /data-code="npm create farm@latest"/);
  assert.match(markup, /data-code="yarn create farm@latest"/);
  assert.equal((markup.match(/data-command-tool=/g) ?? []).length, 2);
  assert.doesNotMatch(markup, / hidden=/);
});
