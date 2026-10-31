import assert from 'node:assert/strict';
import test from 'node:test';
import fs from 'node:fs';
import path from 'node:path';
import { generatedCoverage, summarizeGraph, graphSourcePaths } from '../examples/commerce-lab/benchmark/graph-summary.mjs';

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
  const sampler = read('./performance-report.mjs');
  assert.match(sampler, /measurementCohorts/);
  assert.match(sampler, /for \(const cacheMode of \['cold', 'warm'\]\)/);
  assert.doesNotMatch(sampler, /vite|VITE|sample.buildMs/);
  assert.match(sampler, /copyFile\(path.join\(sourceRoot, 'tsconfig.base.json'\)/);
  assert.match(sampler, /supervision.durationMs/);
  assert.match(sampler, /report.graphs.length === cohorts.length/);
  assert.match(sampler, /report.correctness.length === cohorts.length/);
  const yaml = read('../.github/workflows/ci.yaml').split('\n  performance-report:')[1].split('\n  performance-summary:')[0];
  assert.match(yaml, /os: macos-latest\s+abi: darwin-arm64/); assert.doesNotMatch(yaml, /macos-15-intel|darwin-x64/);
  assert.match(yaml, /performance-python/);
});

test('Mac canonical root aliases resolve absolute and multi-parent graph identities without changing raw IDs', () => {
  const requested = '/var/folders/task/examples/commerce-lab';
  const canonical = '/private/var/folders/task/examples/commerce-lab';
  const generated = `${canonical}/packages/extensions/business-2048`;
  const aliases = [{ from: requested, to: canonical }];
  const relative = path.posix.relative(requested, `${generated}/category-001/Form.tsx`);
  const ids = [relative, `${requested}/packages/extensions/business-2048/index.ts?query`, 'virtual:channel'];
  const original = [...ids];
  const coverage = generatedCoverage(['index.ts', 'category-001/Form.tsx'], { processed: ids }, generated, canonical, { aliases });
  assert.equal(coverage.generatedCovered, 2); assert.deepEqual(coverage.missing, []);
  assert.equal(coverage.categories.processed.query, 1); assert.equal(coverage.categories.processed.virtual, 1);
  assert.equal(coverage.reason, null); assert.deepEqual(ids, original);
  const graph = { modules: ids.map(id => ({ id })), edges: {} };
  assert.equal(summarizeGraph(graph, { projectRoot: canonical, aliases }).applicationModules, 2);
  assert.deepEqual(graph.modules.map(row => row.id), original);
  assert.ok(graphSourcePaths(relative, canonical, aliases).includes(`${generated}/category-001/Form.tsx`));
});

test('canonical coverage uses actual roots, not private-prefix stripping or matching suffixes', () => {
  const root = '/private/var/folders/task/lab', generated = `${root}/packages/extensions/business-2048`;
  const result = generatedCoverage(['index.ts'], { retained: ['/tmp/unrelated/packages/extensions/business-2048/index.ts'] }, generated, root);
  assert.equal(result.generatedCovered, 0); assert.deepEqual(result.missing, ['index.ts']);
  assert.match(result.reason, /absent after canonical-root/);
  assert.equal(result.pathNormalization.outsideGeneratedIdentities, 1);
  assert.equal(generatedCoverage(['index.ts'], { processed: ['/var/folders/task/lab/packages/extensions/business-2048/index.ts'] }, generated, root).generatedCovered, 0);
});

test('Windows drive and normalized parent paths preserve query identity but reject another drive', () => {
  const root = 'C:/work/lab', generated = `${root}/packages/extensions/business-2048`;
  const ids = ['c:\\work\\lab\\packages\\extensions\\business-2048\\category-001\\..\\index.ts?worker', 'D:/work/lab/packages/extensions/business-2048/extra.ts'];
  const result = generatedCoverage(['index.ts', 'extra.ts'], { retained: ids }, generated, root);
  assert.equal(result.generatedCovered, 1); assert.deepEqual(result.missing, ['extra.ts']);
  assert.equal(result.categories.retained.query, 1);
  assert.throws(() => generatedCoverage(['C:/outside.ts'], {}, generated, root), /Invalid expected/);
});

test('Farm graph worker canonicalizes both roots before config resolution and keeps actual raw IDs', () => {
  const source = fs.readFileSync(new URL('./performance-graph-worker.mjs', import.meta.url), 'utf8');
  assert.ok(source.indexOf('const lab = await fs.realpath(inputRoot)') < source.indexOf('await resolveConfig'));
  assert.ok(source.indexOf('const generatedRoot = await fs.realpath(generatedInputRoot)') < source.indexOf('await resolveConfig'));
  assert.match(source, /observed.add\(param.moduleId\)/);
  assert.match(source, /summarizeGraph\(graph, \{ projectRoot: lab, aliases \}\)/);
  assert.match(source, /generatedRoot, lab, \{ aliases \}/);
  assert.match(source, /coverage.reason/);
  assert.doesNotMatch(source, /replace\([^\n]*private|endsWith\(/);
});
