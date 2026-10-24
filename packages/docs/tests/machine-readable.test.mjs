import assert from 'node:assert/strict';
import { createHash, webcrypto } from 'node:crypto';
import fs from 'node:fs/promises';
import { createRequire } from 'node:module';
import { test } from 'node:test';
import { pathToFileURL } from 'node:url';

const require = createRequire(new URL('../package.json', import.meta.url));
const ts = require('typescript');
async function sourceUrl(relative, aliases = {}) {
  const code = await fs.readFile(new URL(relative, import.meta.url), 'utf8');
  const { outputText, diagnostics } = ts.transpileModule(code, {
    compilerOptions: { module: ts.ModuleKind.ES2022, target: ts.ScriptTarget.ES2022, jsx: ts.JsxEmit.ReactJSX },
    fileName: relative,
    reportDiagnostics: true
  });
  assert.equal(diagnostics?.length ?? 0, 0);
  const parsed = ts.createSourceFile('module.js', outputText, ts.ScriptTarget.ES2022, true, ts.ScriptKind.JS);
  const transformed = ts.transform(parsed, [context => root => {
    function visit(node) {
      if (ts.isImportDeclaration(node) && ts.isStringLiteral(node.moduleSpecifier) && aliases[node.moduleSpecifier.text]) {
        return ts.factory.updateImportDeclaration(node, node.modifiers, node.importClause, ts.factory.createStringLiteral(aliases[node.moduleSpecifier.text]), node.attributes);
      }
      return ts.visitEachChild(node, visit, context);
    }
    return ts.visitNode(root, visit);
  }]);
  try {
    const text = ts.createPrinter().printFile(transformed.transformed[0]);
    return `data:text/javascript;base64,${Buffer.from(text).toString('base64')}`;
  } finally { transformed.dispose(); }
}
const source = async (relative, aliases) => import(await sourceUrl(relative, aliases));
const { validateMachineReadableResources: validate } = await source('../src/machine-readable.ts');
const { isDocumentActionUrl, readVerifiedMarkdown } = await source('../../docs-theme-default/src/document-actions-data.ts');
const React = require('react');
const { renderToStaticMarkup } = require('react-dom/server');
const { DocumentActions } = await source('../../docs-theme-default/src/document-actions.tsx', {
  react: pathToFileURL(require.resolve('react')).href,
  'react/jsx-runtime': pathToFileURL(require.resolve('react/jsx-runtime')).href,
  './document-actions-data.js': await sourceUrl('../../docs-theme-default/src/document-actions-data.ts')
});
const snapshotId = 'a'.repeat(64);
const digest = bytes => createHash('sha256').update(bytes).digest('hex');

function fixture() {
  const markdown = Buffer.from('# Setup\n\n## Commands\n\n### npm\n\n```sh\nnpm run dev\n```\n\n### pnpm\n\n```sh\npnpm run dev\n```\n');
  const output = { key: 'v2:en:setup', markdownPath: 'docs/setup/index.md', markdownUrl: '/guide/docs/setup/index.md', sha256: digest(markdown) };
  const page = { key: output.key, title: 'Setup', locale: 'en', version: 'v2', canonicalUrl: 'https://example.com/guide/docs/setup/', markdownUrl: output.markdownUrl, markdownSha256: output.sha256, headings: [{ id: 'commands', title: 'Commands', depth: 2 }] };
  const summary = { schemaVersion: 1, snapshotId, manifestPath: 'docs-ai-manifest.json', indexPath: 'llms.txt', pages: [output] };
  const manifest = { schemaVersion: 1, snapshotId, publicationStatus: 'prerelease', pages: [{ ...output, title: page.title, locale: page.locale, version: page.version, canonicalUrl: page.canonicalUrl, headings: page.headings }] };
  const resources = { 'docs-ai-manifest.json': Buffer.from(JSON.stringify(manifest)), 'llms.txt': Buffer.from('# Example\n\n- [Setup](/guide/docs/setup/index.md)\n'), [output.markdownPath]: markdown };
  const context = { snapshotId, basePath: '/guide/', enabled: true, publicationStatus: 'prerelease', pages: [page] };
  return { summary, manifest, resources, context, output, page, markdown };
}
const run = f => validate(f.summary, f.context, f.resources, Object.keys(f.resources));
const saveManifest = f => { f.resources['docs-ai-manifest.json'] = Buffer.from(JSON.stringify(f.manifest)); };

test('accepts exact native Markdown identities and complete public metadata', () => {
  const f = fixture();
  // Native serde_json Value objects may order keys differently from the plan struct.
  f.manifest.pages[0].headings = [{ depth: 2, id: 'commands', title: 'Commands' }];
  saveManifest(f);
  assert.equal(run(f).snapshotId, snapshotId);
  assert.equal(run(f).pages[0].sha256, digest(f.markdown));
});

test('AI output is opt-in and enabled builds cannot omit machine output', () => {
  const f = fixture();
  assert.equal(validate(undefined, { ...f.context, enabled: false, pages: [] }, {}, []), undefined);
  assert.throws(() => validate(undefined, f.context, {}, []), /enabled output is missing/);
  f.context.enabled = false;
  assert.throws(() => run(f), /explicit AI configuration/);
});

test('rejects changed Markdown bytes, stale snapshots and unlisted resources', () => {
  const changed = fixture(); changed.resources[changed.output.markdownPath] = Buffer.from('different deployment');
  assert.throws(() => run(changed), /bytes differ/);
  const stale = fixture(); stale.summary.snapshotId = 'b'.repeat(64);
  assert.throws(() => run(stale), /snapshot/);
  const publicStale = fixture(); publicStale.manifest.snapshotId = 'b'.repeat(64); saveManifest(publicStale);
  assert.throws(() => run(publicStale), /snapshot/);
  const absent = fixture();
  assert.throws(() => validate(absent.summary, absent.context, absent.resources, ['docs-ai-manifest.json', 'llms.txt']), /missing/);
});

test('rejects path escapes before URL normalization and page/hash disagreement', () => {
  for (const path of ['../secret.md', '/secret.md', 'C:/secret.md', 'docs/%2e%2e/secret.md', 'docs/%2fsecret.md', 'docs\\secret.md', 'docs//secret.md']) {
    const f = fixture(); f.summary.pages[0].markdownPath = path;
    assert.throws(() => run(f), /invalid page identity/);
  }
  const f = fixture(); f.context.pages[0].markdownSha256 = 'f'.repeat(64);
  assert.throws(() => run(f), /identities differ/);
});

test('rejects public source paths and metadata drifting from HTML identity', () => {
  const privateField = fixture(); privateField.manifest.sourcePath = 'C:/private/source'; saveManifest(privateField);
  assert.throws(() => run(privateField), /public manifest fields/);
  for (const change of [page => { page.locale = 'zh'; }, page => { page.version = 'v1'; }, page => { page.canonicalUrl = 'https://elsewhere.example/'; }, page => { page.headings[0].title = 'Different heading'; }]) {
    const f = fixture(); f.manifest = structuredClone(f.manifest); change(f.manifest.pages[0]); saveManifest(f);
    assert.throws(() => run(f), /metadata differs/);
  }
});

test('rejects missing pages, duplicate AI paths and non-UTF8 public text', () => {
  const missing = fixture(); missing.summary.pages = [];
  assert.throws(() => run(missing), /missing from output/);
  const duplicate = fixture(); duplicate.summary.pages.push({ ...duplicate.output });
  assert.throws(() => run(duplicate), /invalid page identity/);
  const invalid = fixture(); invalid.resources['llms.txt'] = Buffer.from([0xff]);
  assert.throws(() => run(invalid), /UTF-8/);
});

test('document action URLs reject remote origins and traversal', () => {
  for (const url of ['https://example.com/file.md', '//example.com/file.md', '/a/../file.md', '/%2e%2e/file.md', '/a%2ffile.md', '/a\\file.md', '/a?secret=1', '/a#fragment', '/a//file.md']) assert.equal(isDocumentActionUrl(url), false, url);
  assert.equal(isDocumentActionUrl('/guide/docs/setup/index.md'), true);
  assert.equal(isDocumentActionUrl('/zh/指南/index.md'), true);
});

test('copy retrieval preserves exact source text and uses same-origin nonredirecting transport', async () => {
  const bytes = Buffer.from('```js\nconsole.log("中文");\n```\n');
  let request;
  const text = await readVerifiedMarkdown('/guide/setup/index.md', digest(bytes), {
    fetch: async (url, options) => { request = { url, options }; return new Response(bytes, { headers: { 'Content-Type': 'text/markdown; charset=utf-8' } }); },
    digest: webcrypto.subtle.digest.bind(webcrypto.subtle)
  });
  assert.equal(text, bytes.toString());
  assert.equal(request.options.redirect, 'error');
  assert.equal(request.options.credentials, 'same-origin');
  assert.equal(request.options.headers.Accept, 'text/markdown');
});

test('copy retrieval rejects new-deployment text and HTML/error responses', async () => {
  const bytes = Buffer.from('# Current document\n');
  const dependencies = { fetch: async () => new Response('# Different document\n', { headers: { 'Content-Type': 'text/markdown' } }), digest: webcrypto.subtle.digest.bind(webcrypto.subtle) };
  await assert.rejects(readVerifiedMarkdown('/guide/setup/index.md', digest(bytes), dependencies), /content changed/);
  dependencies.fetch = async () => new Response('<h1>Not found</h1>', { headers: { 'Content-Type': 'text/html' } });
  await assert.rejects(readVerifiedMarkdown('/guide/setup/index.md', digest(bytes), dependencies), /Markdown response/);
  dependencies.fetch = async () => new Response('Not found', { status: 404 });
  await assert.rejects(readVerifiedMarkdown('/guide/setup/index.md', digest(bytes), dependencies), /Markdown response/);
});

test('SSR exposes only declared document capabilities with accessible translated labels', () => {
  const render = props => renderToStaticMarkup(React.createElement(DocumentActions, props));
  assert.equal(render({}), '');
  assert.equal(render({ markdownUrl: '/guessed/index.md' }), '');
  assert.equal(render({ markdownUrl: '//remote.example/index.md', markdownSha256: 'a'.repeat(64) }), '');
  const guide = render({ agentGuideUrl: '/agents/' });
  assert.match(guide, /href="\/agents\/"/);
  assert.doesNotMatch(guide, /Copy Markdown|View Markdown/);
  const markup = render({ markdownUrl: '/zh/docs/setup/index.md', markdownSha256: 'a'.repeat(64), agentGuideUrl: '/zh/agents/', labels: { region: '文档格式', copy: '复制 Markdown', view: '查看 Markdown', guide: 'Agent 指南' } });
  assert.match(markup, /aria-label="文档格式"/);
  assert.match(markup, /<button type="button">复制 Markdown<\/button>/);
  assert.match(markup, /type="text\/markdown">查看 Markdown/);
  assert.match(markup, /href="\/zh\/agents\/">Agent 指南/);
  assert.match(markup, /role="status" aria-live="polite"/);
  assert.doesNotMatch(markup, /Markdown copied|SKILL|chat/i);
});
