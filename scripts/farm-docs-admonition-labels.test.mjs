import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {test} from 'node:test';
import {resolveConfig, createCompiler} from '../packages/core/dist/index.js';
import docs from '../rust-plugins/docs/scripts/func.js';

async function fixture(run) {
  const parent = path.resolve('examples/docs-minimal');
  const root = await fs.mkdtemp(path.join(parent, 'admonition-labels-'));
  try {
    await fs.mkdir(path.join(root, 'en'));
    await fs.mkdir(path.join(root, 'zh'));
    return await run(root);
  } finally {
    assert.equal(path.dirname(root), parent);
    assert.ok(path.basename(root).startsWith('admonition-labels-'));
    await fs.rm(root, {recursive: true, force: true});
  }
}
async function prepare(root, collections) {
  const compiler = createCompiler(await resolveConfig({
    root, configFile: false,
    compilation: {input: {site: 'docs:site'}, output: {path: 'output', targetEnv: 'node', format: 'cjs'}, persistentCache: false, minify: false, sourcemap: false, lazyCompilation: false},
    plugins: [docs({site: {title: 'Labels', url: 'https://example.com', locales: {default: 'en', items: ['en', 'zh']}, docs: {collections}}, prepare: true})]
  }, 'build', 'production'));
  await compiler.compile();
  return JSON.parse(compiler.resources()['docs-plan.json'].toString());
}

test('collection labels change only implicit titles and preserve explicit or empty titles', async () => {
  await fixture(async root => {
    await fs.writeFile(path.join(root, 'zh/index.md'), '# Labels\n\n:::note\nDefault body.\n\n:::tip\nNested body.\n:::\n:::\n\n:::note note\nExplicit body.\n:::\n\n:::note {title=""}\nEmpty explicit.\n:::\n\n:::warning\nHidden default.\n:::\n');
    const plan = await prepare(root, [{id: 'zh', dir: 'zh', routeBase: '/zh/docs/', locale: 'zh', admonitionLabels: {note: '备注', tip: '提示', warning: ''}}]);
    const code = plan.modules[0].code;
    assert.equal((code.match(/farm-admonition-title/g) ?? []).length, 3);
    for (const text of ['备注', '提示', 'Default body.', 'Nested body.', 'Explicit body.', 'Empty explicit.', 'Hidden default.']) assert.ok(code.includes(text), text);
    assert.ok(code.includes('"note"') || code.includes("'note'"), 'explicit same-as-kind title remains authored');
    assert.ok(!JSON.stringify(plan.site).includes('admonitionLabels'), 'compile-only labels remain private');
  });
});

test('fallback content uses target collection labels without mutating the source collection', async () => {
  await fixture(async root => {
    await fs.writeFile(path.join(root, 'en/index.md'), '# Fallback\n\n:::note\nOriginal author body.\n:::\n');
    const plan = await prepare(root, [
      {id: 'en', group: 'docs', dir: 'en', locale: 'en', routeBase: '/docs/', admonitionLabels: {note: 'Source label'}},
      {id: 'zh', group: 'docs', dir: 'zh', locale: 'zh', fallbackLocale: 'en', routeBase: '/zh/docs/', admonitionLabels: {note: '目标标签'}}
    ]);
    for (const [collection, expected, forbidden] of [['en', 'Source label', '目标标签'], ['zh', '目标标签', 'Source label']]) {
      const page = plan.routes.find(route => route.page.collection === collection).page;
      const code = plan.modules.find(module => module.key === page.key).code;
      assert.ok(code.includes(expected), collection + ' default label');
      assert.ok(!code.includes(forbidden), collection + ' labels do not leak across collections');
      assert.ok(code.includes('Original author body.'));
    }
  });
});

test('collection labels reject unknown kinds and non-string values', async () => {
  await fixture(async root => {
    await fs.writeFile(path.join(root, 'en/index.md'), '# Invalid labels\n');
    for (const admonitionLabels of [{unsupported: 'Wrong'}, {note: 42}, []]) {
      await assert.rejects(prepare(root, [{id: 'en', dir: 'en', routeBase: '/docs/', admonitionLabels}]), /DOCS_CONFIG/);
    }
  });
});
