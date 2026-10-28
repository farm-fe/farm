import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { aggregate, compatible, command, workloadFingerprint, businessFixture, workloadFileCounts, workloadScale } from './performance-report.mjs';

test('failed and missing samples do not become zero-duration measurements', () => {
  const cells = aggregate([
    { scored: false, cacheMode: 'cold', success: true, durationMs: 999 },
    { scored: true, cacheMode: 'cold', success: true, durationMs: 10 },
    { scored: true, cacheMode: 'cold', success: true, durationMs: 30 },
    { scored: true, cacheMode: 'cold', success: false, durationMs: null }
  ]);
  assert.equal(cells[0].medianMs, 20);
  assert.equal(cells[0].iqrMs, 10);
  assert.equal(cells[0].validSamples, 2);
  assert.equal(cells[1].medianMs, null);
});
test('baseline requires matching explicit protocol, fixture and toolchain dimensions', () => {
  const metadata = { protocol: 'v1', fixtureSha256: 'abc', harnessSha256: 'harness-a', node: '22', pnpm: '9', os: 'linux', arch: 'x64', lockSha256: 'def' };
  assert.ok(compatible(metadata, { ...metadata }));
  assert.equal(compatible(metadata, { ...metadata, arch: 'arm64' }), false);
  assert.equal(compatible(metadata, { ...metadata, harnessSha256: 'harness-b' }), false);
  assert.equal(compatible({}, {}), false);
});
test('business-2048 generates real business files, never reports files as modules', async () => {
  const fixture = businessFixture();
  assert.equal(workloadScale, 'business-2048');
  assert.equal(Object.keys(fixture).length, 2048 * 14 + 1);
  assert.ok(Object.keys(fixture).some(name => name.startsWith('category-2048/')));
  const imports = [...fixture['index.ts'].matchAll(/from '\.\/([^']+)'/g)].map(match => match[1]);
  assert.equal(imports.length, 2048);
  assert.equal(new Set(imports).size, 2048);
  for (const target of imports) assert.ok(fixture[target + '.tsx'], target);
  assert.match(fixture['category-2048/Form.tsx'], /import/);
  assert.ok(fixture['category-2048/business.ts']);
  const reached = new Set();
  function visit(file) {
    if (reached.has(file)) return;
    reached.add(file);
    for (const match of fixture[file].matchAll(/(?:from\s+|import\s*)['"]([^'"]+)['"]/g)) {
      if (!match[1].startsWith('.')) continue;
      const base = path.posix.normalize(path.posix.join(path.posix.dirname(file), match[1]));
      if (base.startsWith('../')) continue; // Existing benchmark helper, outside generated fixture.
      const resolved = [base, base + '.ts', base + '.tsx', base + '.js', base + '.jsx'].find(name => Object.hasOwn(fixture, name));
      assert.ok(resolved, `${file}: ${match[1]}`);
      visit(resolved);
    }
  }
  visit('index.ts');
  assert.equal(reached.size, Object.keys(fixture).length, 'Every generated business input is statically reachable');
  const counts = await workloadFileCounts(fileURLToPath(new URL('../', import.meta.url)), fixture);
  assert.equal(counts.generatedFiles, 28673);
  assert.equal(counts.baseSourceFiles, 240);
  assert.equal(counts.totalSourceFiles, 28913);
  assert.equal(counts.fileCountIsNotModuleCount, true);
});

test('identity and raw resource evidence precede guard; source restoration and Vite scope remain explicit', async () => {
  const source = await fs.readFile(new URL('./performance-report.mjs', import.meta.url), 'utf8');
  assert.ok(source.indexOf('artifactProducerSHA:') < source.indexOf('assertPerformanceResources(report.resources)'));
  assert.ok(source.indexOf('report.resources = resourceSnapshot') < source.indexOf('assertPerformanceResources(report.resources)'));
  assert.match(source, /report\.resources = resourceSnapshot[^;]+; await persist\(\);/);
  assert.match(source, /sourcesUnchanged = before === await trackedFingerprint/);
  assert.match(source, /catalog-quote-subset/);
  assert.match(source, /COMMERCE_SCALE: workloadScale/);
});
test('fingerprint includes the actual Commerce apps, config and business packages', async () => {
  const root = fileURLToPath(new URL('../', import.meta.url));
  const first = await workloadFingerprint(root, { generated: 'fixture' });
  assert.match(first, /^[a-f0-9]{64}$/);
  assert.notEqual(first, await workloadFingerprint(root, { generated: 'different fixture' }));
});
test('failed child command and timeout cleanup status are explicit', async t => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-perf-test-'));
  try {
    await assert.rejects(command(process.execPath, ['-e', 'process.exit(7)'], { log: path.join(directory, 'error.log') }), /Command failed/);
    let timedOut;
    try {
      // Self-expiring fixture protects the test even if OS tree termination fails.
      await command(process.execPath, ['-e', 'setTimeout(() => process.exit(0), 1500)'], { log: path.join(directory, 'timeout.log'), timeoutMs: 100 });
      assert.fail('Expected timeout');
    } catch (error) {
      timedOut = error; assert.equal(error.timedOut, true); assert.ok(Number.isInteger(error.pid));
      t.diagnostic(JSON.stringify({ timeout: true, cleanupConfirmed: error.cleanupConfirmed, cleanupError: error.cleanupError ?? null }));
    }
    finally {
      if (timedOut?.pid) {
        const deadline = Date.now() + 4000;
        let alive = true;
        while (alive && Date.now() < deadline) {
          try { process.kill(timedOut.pid, 0); } catch (error) { if (error.code === 'ESRCH') alive = false; else throw error; }
          if (alive) await new Promise(resolve => setTimeout(resolve, 25));
        }
        assert.equal(alive, false, 'Self-expiring owned fixture must leave no process');
      }
    }
  } finally { await fs.rm(directory, { recursive: true }); }
});

test('workload fingerprint changes for HTML, adapters and scale inputs, not generated output', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-perf-inputs-'));
  const lab = path.join(root, 'examples/commerce-lab');
  try {
    const inputs = ['index.html', 'adapters/vite.mjs', 'benchmark/scale-profiles.mjs', 'packages/business/index.ts'];
    for (const file of inputs) { await fs.mkdir(path.dirname(path.join(lab, file)), { recursive: true }); await fs.writeFile(path.join(lab, file), 'original'); }
    const original = await workloadFingerprint(root, {});
    for (const file of inputs) {
      await fs.writeFile(path.join(lab, file), 'mutated');
      assert.notEqual(await workloadFingerprint(root, {}), original, file);
      await fs.writeFile(path.join(lab, file), 'original');
    }
    for (const file of ['dist/output.js', '.cache/result', 'packages/extensions/baseline/index.ts']) {
      await fs.mkdir(path.dirname(path.join(lab, file)), { recursive: true }); await fs.writeFile(path.join(lab, file), 'generated');
    }
    assert.equal(await workloadFingerprint(root, {}), original);
  } finally { await fs.rm(root, { recursive: true }); }
});
test('performance job is non-blocking, follows Examples and pins artifact names to SHA', async () => {
  const source = await fs.readFile(new URL('../.github/workflows/ci.yaml', import.meta.url), 'utf8');
  const job = source.split('\n  performance-report:')[1].split('\n  performance-summary:')[0];
  const dependencies = job.match(/needs:\s*\[([^\]]+)\]/)?.[1].split(',').map(name => name.trim());
  assert.deepEqual(new Set(dependencies), new Set(['examples-test', 'docs-acceptance', 'ts-test', 'type-check', 'check-native-candidate-assembly']));
  assert.match(job, /continue-on-error: true/);
  assert.match(job, /timeout-minutes: 20/);
  assert.equal((job.match(/os: .*latest/g) ?? []).length, 3);
  assert.doesNotMatch(job, /cargo |pattern:/);
  for (const line of job.split('\n').filter(line => line.trim().startsWith('name:') && line.includes('plugin-'))) assert.ok(line.includes('github.sha'));
  assert.match(job, /playwright-chromium\/cli.js install --with-deps chromium/);
  assert.match(source.split('\n  performance-contract-tests:')[1].split('\n  call-rust-build-user:')[0], /node --test scripts\/performance\*\.test\.mjs/);
  const sampler = await fs.readFile(new URL('./performance-report.mjs', import.meta.url), 'utf8');
  assert.ok(sampler.indexOf('performance-correctness-worker.mjs') > sampler.indexOf('report.vite.status ='));
});
test('setup failure still produces invalid structured evidence', async () => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-perf-evidence-'));
  try {
    const script = fileURLToPath(new URL('./performance-evidence.mjs', import.meta.url));
    for (const stage of ['init', 'finalize']) await command(process.execPath, [script, stage], { cwd: directory, log: path.join(directory, 'test.log') });
    const report = JSON.parse(await fs.readFile(path.join(directory, 'performance-results/raw.json'), 'utf8'));
    assert.equal(report.status, 'invalid');
    assert.equal(report.publishable, false);
    assert.deepEqual(report.records, []);
    for (const file of ['summary.json', 'samples.csv', 'run.log']) await fs.access(path.join(directory, 'performance-results', file));
  } finally { await fs.rm(directory, { recursive: true }); }
});
