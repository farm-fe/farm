import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { command } from './performance-report.mjs';
const root = fileURLToPath(new URL('../', import.meta.url));
export const docsSuites = Object.freeze({
  source: [...['registry-consumer', 'example-prerequisites', 'main-deploy-gate', 'vercel-build', 'native-guard', 'ecosystem', 'header-contract', 'reading-ui-contract', 'navigation-events', 'search-layer', 'search-provider', 'sidebar-sections', 'suites'].map(name => `scripts/farm-docs-${name}.test.mjs`), 'scripts/example-native-inputs.test.mjs', 'scripts/native-artifact-inventory.test.mjs', 'scripts/local-docs-cleanup.test.mjs'],
  distribution: ['package-contract', 'consumer-isolation', 'local-registry', 'distribution-chain', 'target-closure'].map(name => `scripts/farm-docs-${name}.test.mjs`),
  platform: ['scripts/farm-docs-platform-preparation.test.mjs'],
  compiled: ['scripts/farm-docs-cli-diagnostics.test.mjs', 'scripts/farm-docs-config-alias.test.mjs', 'packages/docs/tests/build-cache.test.mjs'],
  browser: ['full-list-theme', 'admonition-labels', 'blog-list-content', 'theme', 'runtime'].map(name => `scripts/farm-docs-${name}.test.mjs`),
  diagrams: ['scripts/check-doc-diagrams.test.mjs'],
  'notices-source': ['scripts/farm-docs-notices.test.mjs'],
  'notices-built': ['scripts/farm-docs-notices.test.mjs'],
  'archive-python': ['scripts/farm-docs-extract-package.test.py']
});
export const importedCoverage = { 'farm-docs-theme-presentation.test.mjs': 'farm-docs-theme.test.mjs' };
export async function auditDocsSuites(directory = root) {
  const classified = new Set(Object.values(docsSuites).flat().map(file => path.basename(file)));
  for (const [file, importer] of Object.entries(importedCoverage)) {
    assert.ok(classified.has(importer));
    assert.ok((await fs.readFile(path.join(directory, 'scripts', importer), 'utf8')).includes(`import './${file}'`), 'Indirect suite coverage changed');
    classified.add(file);
  }
  const actual = (await fs.readdir(path.join(directory, 'scripts'))).filter(file => /^farm-docs-.*\.test\.mjs$/.test(file));
  const missing = actual.filter(file => !classified.has(file));
  assert.deepEqual(missing, [], 'Every retained Docs test needs an explicit semantic suite');
  for (const file of Object.values(docsSuites).flat()) await fs.access(path.join(directory, file));
  return { classified: actual.length, groups: Object.keys(docsSuites), indirectCoverage: importedCoverage };
}
export function docsSuiteCommand(name) {
  const files = docsSuites[name]; assert.ok(files, `Unknown Docs suite: ${name}`);
  if (name === 'archive-python') return { executable: process.env.FARM_DOCS_TEST_PYTHON ?? 'python', args: files };
  if (name.startsWith('notices-')) return { executable: process.execPath, args: [...files, ...(name === 'notices-source' ? ['--source-only'] : [])] };
  if (name === 'browser') return { executable: process.execPath, args: ['scripts/farm-docs-run-guarded.mjs', ...files] };
  return { executable: process.execPath, args: [...(name === 'platform' ? ['--experimental-strip-types'] : []), '--test', '--test-concurrency=1', ...files] };
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const suite = process.argv[2];
  if (suite === 'audit') console.log(JSON.stringify(await auditDocsSuites()));
  else {
    if (suite === 'browser') assert.ok(process.env.CODEX_DOCS_NATIVE_BASELINE, 'Browser/native acceptance requires the existing provenance guard');
    const { executable, args } = docsSuiteCommand(suite);
    const logs = await fs.mkdtemp(path.join(process.env.RUNNER_TEMP ?? os.tmpdir(), 'farm-docs-suite-'));
    const log = path.join(logs, `${suite}.log`);
    console.log(`Docs suite ${suite}; log ${log}`);
    try { await command(executable, args, { cwd: root, env: process.env, log, timeoutMs: suite === 'browser' ? 35 * 60 * 1000 : 5 * 60 * 1000 }); }
    catch (error) { console.error((await fs.readFile(log, 'utf8')).slice(-12000)); throw error; }
    console.log((await fs.readFile(log, 'utf8')).slice(-2000));
  }
}
