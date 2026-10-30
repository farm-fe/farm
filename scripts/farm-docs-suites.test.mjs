import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import vm from 'node:vm';
import { auditDocsSuites, docsSuiteCommand, docsSuites } from './farm-docs-suites.mjs';

test('workflow exports real output newline and rejects CR/LF without rejecting guard path letters', async () => {
  const source = await fs.readFile(new URL('../.github/workflows/farm-docs.yaml', import.meta.url), 'utf8');
  const start = source.indexOf('          const nodeOptions=');
  const end = source.indexOf('\n          NODE', start);
  assert.ok(start >= 0 && end > start);
  const code = source.slice(start, end);
  function execute(options) {
    const writes = [];
    vm.runInNewContext(code, {
      process: { env: { NODE_OPTIONS: options, GITHUB_OUTPUT: 'output' } },
      path: { resolve: value => '/repo/' + value },
      pathToFileURL: value => ({ href: 'file://' + value }),
      fs: { appendFileSync: (file, value) => writes.push({ file, value }) }
    });
    return writes;
  }
  const writes = execute('--trace-warnings');
  assert.equal(writes.length, 1);
  assert.equal(writes[0].file, 'output');
  assert.equal(writes[0].value, 'node_options=--trace-warnings --import=file:///repo/scripts/farm-docs-native-guard.mjs\n');
  for (const options of ['--trace-warnings\nBAD=value', '--trace-warnings\rBAD=value']) assert.throws(() => execute(options), /single line/);
});
test('all retained Docs tests belong to explicit source/distribution/compiled/browser groups', async () => {
  const audit = await auditDocsSuites(); assert.ok(audit.classified >= 27);
  assert.ok(docsSuites.source.every(file => !docsSuites.browser.includes(file)));
  assert.ok(docsSuiteCommand('browser').args.includes('scripts/farm-docs-run-guarded.mjs'));
  assert.ok(docsSuiteCommand('platform').args.includes('--experimental-strip-types'));
  assert.equal(docsSuiteCommand('archive-python').args[0], 'scripts/farm-docs-extract-package.test.py');
  assert.throws(() => docsSuiteCommand('all'), /Unknown Docs suite/);
});
test('E2E calls Docs, keeps both OS lanes and release reference until actual compatibility validation', async () => {
  const ci = await fs.readFile(new URL('../.github/workflows/ci.yaml', import.meta.url), 'utf8');
  const docs = await fs.readFile(new URL('../.github/workflows/farm-docs.yaml', import.meta.url), 'utf8');
  assert.match(ci, /uses: \.\/\.github\/workflows\/farm-docs.yaml/);
  assert.match(docs, /workflow_call:/); assert.doesNotMatch(docs, /^  pull_request:/m);
  assert.match(docs, /os: \[windows-latest, ubuntu-24.04\]/);
  assert.match(docs, /run: pnpm --dir packages\/core run build:rs/);
  assert.match(docs, /cargo test .*farmfe_docs_snapshot/);
  assert.match(docs, /CODEX_DOCS_NATIVE_BASELINE/);
  const platform = await fs.readFile(new URL('../.github/workflows/ci-primary-platform.yaml', import.meta.url), 'utf8');
  const examples = platform.split('\n  examples-test:')[1].split('\n  ts-test:')[0];
  for (const plugin of ['ssr-css', 'syntax-highlight', 'docs', 'docs-snapshot', 'docs-seo', 'docs-snippets']) assert.ok(examples.includes(`-plugin-${plugin}`));
  assert.match(examples, /FARM_EXAMPLE_PREBUILT_ONLY: '1'/);
});
