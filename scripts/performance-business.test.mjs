import assert from 'node:assert/strict';
import test from 'node:test';
import fs from 'node:fs';
import { generatedCoverage } from '../examples/commerce-lab/benchmark/graph-summary.mjs';

test('actual POSIX/Windows graph paths cover generated sources without counting virtual/query nodes as extra files', () => {
  for (const root of ['/tmp/lab', 'C:\\lab']) {
    const generated = root + '/packages/extensions/business-2048';
    const result = generatedCoverage(['index.ts', 'category-001/Form.tsx', 'category-001/style.scss', 'category-001/icon.svg'], {
      loaded: [generated + '/index.ts', generated + '/category-001/Form.tsx', generated + '/category-001/style.scss?css', generated + '/category-001/icon.svg'],
      retained: ['virtual:channel', 'apps/admin/import.ts?worker'], workers: ['/node_modules/worker.js'],
    }, generated, root);
    assert.equal(result.generatedFiles, 4); assert.equal(result.generatedCovered, 4); assert.deepEqual(result.missing, []);
    assert.equal(result.categories.loaded.css, 1); assert.equal(result.categories.loaded.assets, 1);
    assert.equal(result.categories.loaded.query, 1); assert.equal(result.categories.retained.virtual, 1);
    assert.equal(result.categories.workers.identities, 1);
  }
});

test('missing generated inputs stay missing; retained chunks never substitute an inferred total', () => {
  const result = generatedCoverage(['index.ts', 'category-2048/Form.tsx'], { retained: ['packages/extensions/business-2048/index.ts'] }, '/lab/packages/extensions/business-2048', '/lab');
  assert.equal(result.generatedCovered, 1); assert.deepEqual(result.missing, ['category-2048/Form.tsx']);
  assert.throws(() => generatedCoverage(['../outside.ts'], {}, '/lab/generated', '/lab'));
});

test('main performance protocol runs the complete original app, guarded pairs and unscored coverage/correctness', () => {
  const read = relative => fs.readFileSync(new URL(relative, import.meta.url), 'utf8');
  const config = read('../examples/commerce-lab/benchmark/vite-business.mjs'), sampler = read('./performance-report.mjs');
  assert.match(config, /target: 'esnext', sourcemap: true, minify: 'esbuild'/);
  assert.match(config, /input: path.join\(root, 'index.html'\)/); assert.match(config, /cache: false/);
  assert.match(config, /generated.length, 28673/); assert.match(config, /level: 6/);
  assert.match(config, /transformWithEsbuild\(jsx/);
  assert.match(sampler, /measurementCohorts/); assert.match(sampler, /for \(const cacheMode of \['cold', 'warm'\]\)/);
  assert.match(sampler, /warmComparable = false/);
  assert.doesNotMatch(sampler, /prepareComparisonFixture|vite-subset|sample.buildMs/);
  assert.match(sampler, /copyFile\(path.join\(sourceRoot, 'tsconfig.base.json'\)/);
  assert.match(sampler, /supervision.durationMs/);
  const yaml = read('../.github/workflows/ci.yaml').split('\n  performance-report:')[1].split('\n  performance-summary:')[0];
  assert.match(yaml, /os: macos-latest\s+abi: darwin-arm64/); assert.doesNotMatch(yaml, /macos-15-intel|darwin-x64/);
  assert.match(yaml, /performance-python/);
});
