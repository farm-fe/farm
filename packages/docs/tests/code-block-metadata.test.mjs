import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import { test } from 'node:test';
const require = createRequire(new URL('../package.json', import.meta.url));
const ts = require('typescript');
const React = require('react');
const { renderToString } = require('react-dom/server');
const loaded = new Map();
function load(name, aliases = {}) {
  const file = new URL(`../src/${name}`, import.meta.url);
  if (loaded.has(file.href) && !Object.keys(aliases).length) return loaded.get(file.href);
  const result = ts.transpileModule(fs.readFileSync(file, 'utf8'), {
    fileName: name, reportDiagnostics: true,
    compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS, jsx: ts.JsxEmit.ReactJSX }
  });
  assert.equal(result.diagnostics?.length ?? 0, 0);
  const module = { exports: {} };
  new Function('require', 'module', 'exports', result.outputText)(specifier => {
    if (Object.hasOwn(aliases, specifier)) return aliases[specifier];
    if (specifier === './code-block-metadata.js') return load('code-block-metadata.ts');
    return require(specifier);
  }, module, module.exports);
  if (!Object.keys(aliases).length) loaded.set(file.href, module.exports);
  return module.exports;
}
const { serializeCodeBlockMetadata: encode, CODE_BLOCK_METADATA_MARKER: marker } = load('code-block-metadata.ts');
const { CodeBlock } = load('code-block.tsx');
const markerAttribute = 'data-farm-docs-code-block';
const owned = (code = 'a\r\nb', extra = '') => `<pre ${markerAttribute}="${marker}" data-code="${code}"${extra}><code>x</code></pre>`;

test('CR encoding is narrow, exact, idempotent and keeps all React escaping', () => {
  const input = owned('&lt;script&gt;&amp;#13;&quot;&#x27;λ\r\nb\rc\n', ' data-title="untouched\r"');
  const expected = owned('&lt;script&gt;&amp;#13;&quot;&#x27;λ&#13;\nb&#13;c\n', ' data-title="untouched\r"');
  assert.equal(encode(input), expected);
  assert.equal(encode(expected), expected);
  assert.equal(encode(owned('a\nb')), owned('a\nb'));
  const mixed = `<PRE class='x>y' DATA-CODE='a>\rb' ${markerAttribute.toUpperCase()}='${marker}'>x</PRE>`;
  assert.equal(encode(mixed), mixed.replace('a>\rb', 'a>&#13;b'));
});

test('comments, raw text, RCDATA, foreign content and unrelated attributes remain unchanged', () => {
  for (const prefix of [`<!--${owned()}-->`, ...['script', 'style', 'textarea', 'title', 'xmp', 'iframe', 'noembed', 'noframes', 'noscript']
    .map(name => `<${name}>${owned()}</${name}>`), `<svg><foreignObject>${owned()}</foreignObject></svg>`, `<math>${owned()}</math>`]) {
    assert.equal(encode(prefix + owned('real\r')), prefix + owned('real&#13;'));
  }
  assert.equal(encode('<plaintext>' + owned()), '<plaintext>' + owned());
  for (const input of ['<pre data-code="a\r">x</pre>', `<div ${markerAttribute}="${marker}" data-code="a\r">x</div>`,
    `<pre ${markerAttribute}="other_marker" data-code="a\r">x</pre>`, `<pre fake-${markerAttribute}="${marker}" data-code="a\r">x</pre>`]) assert.equal(encode(input), input);
});

test('malformed marked output raises a diagnostic instead of returning partial repairs', () => {
  for (const suffix of ['<!-- unclosed', '<script><!--<script></script>' + owned() + '</script>', '<pre a="unclosed', '<!unknown>', '<svg>',
    `<pre ${markerAttribute}="${marker}" data-code="a\r" data-code="b">`,
    `<pre ${markerAttribute}="${marker}" ${markerAttribute}="other" data-code="a\r">`,
    `<pre ${markerAttribute}=${marker} data-code="a\r">`, `<pre ${markerAttribute}="${marker}" data-code=x>`,
    `<pre ${markerAttribute}="${marker}" data-code="x\0y">`]) {
    assert.throws(() => encode(owned('first\r') + suffix), /DOCS_SSR_CODE_METADATA/);
  }
  // Without any relevant CR/marker there is no lexical scan and no new failure.
  assert.equal(encode('<script>unclosed'), '<script>unclosed');
  assert.equal(encode('<script>unclosed\r'), '<script>unclosed\r');
});

test('actual CodeBlock owns its CR marker; caller props cannot forge or remove it', () => {
  const marked = renderToString(React.createElement(CodeBlock, { 'data-code': 'a\r\nb', [markerAttribute]: 'forged' }, React.createElement('code', null, 'text')));
  assert.ok(marked.includes(`${markerAttribute}="${marker}"`));
  assert.equal(marked.includes('forged'), false);
  for (const raw of [undefined, 'LF\nonly', 'NUL\0and\r\n']) {
    const html = renderToString(React.createElement(CodeBlock, { 'data-code': raw, [markerAttribute]: marker }));
    assert.equal(html.includes(markerAttribute), false);
  }
});

test('real mixed SSR preserves the existing NUL fallback without failing the whole page', () => {
  const nul = 'keep\0NUL\r\n';
  const block = raw => React.createElement(CodeBlock, { 'data-code': raw }, React.createElement('code', null, raw));
  const nulHtml = renderToString(block(nul));
  assert.equal(nulHtml.includes(markerAttribute), false);
  assert.equal(encode(nulHtml), nulHtml, 'No claim that HTML can losslessly carry NUL');
  const mixed = renderToString(React.createElement(React.Fragment, null, block('normal\r\n'), block(nul)));
  let encoded;
  assert.doesNotThrow(() => { encoded = encode(mixed); });
  assert.ok(encoded.includes('data-code="normal&#13;\n"'));
  assert.ok(encoded.endsWith(nulHtml), 'Unmarked NUL component keeps prior React/JSX bytes');
  // The low-level reserved protocol remains strict for a forged owned NUL.
  assert.throws(() => encode(owned(nul)), /DOCS_SSR_CODE_METADATA/);
});

test('real React SSR preserves original metadata and caller markup without decoding XSS', () => {
  const raw = '</code><script>alert("x")</script>& λ\r\n';
  const html = renderToString(React.createElement(CodeBlock, { 'data-code': raw, 'data-title': 'safe title' }, React.createElement('code', null, 'display')));
  const encoded = encode(html);
  assert.ok(encoded.includes('data-code="&lt;/code&gt;&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt;&amp; λ&#13;\n"'));
  assert.equal(encoded.includes('<script>'), false);
  assert.equal(encoded, html.replace(/( data-code=")([^"]*)(")/, (_, start, value, end) => start + value.replaceAll('\r', '&#13;') + end));
});

test('server entry applies the encoder to document and not-found SSR boundaries', () => {
  const element = () => React.createElement(CodeBlock, { 'data-code': 'a\r\nb' }, React.createElement('code', null, 'display'));
  const entry = load('server-entry.ts', {
    'docs:application': { snapshotId: 'fixture', renderPage: element, renderNotFound: element },
    './runtime-frame.js': { RuntimeFrame: ({ content }) => content }
  });
  for (const html of [entry.render('one'), entry.renderNotFound('en')]) assert.ok(html.includes('data-code="a&#13;\nb"'));
  assert.equal(entry.snapshotId, 'fixture');
});

test('marker is a reserved convention, not authentication or a sanitizer', () => {
  // Exact authored forgery cannot be distinguished from generated markup by
  // scanning HTML. It only preserves the original CR and never decodes XSS.
  assert.equal(encode(owned('&lt;script&gt;\r')), owned('&lt;script&gt;&#13;'));
});
