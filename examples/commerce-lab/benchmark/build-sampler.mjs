import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawn } from 'node:child_process';
import { performance } from 'node:perf_hooks';
import { randomUUID } from 'node:crypto';
import { buildSchedule } from './build-schedule.mjs';
import { captureEnvironment } from './environment.mjs';
import { verifyCompression } from './artifacts.mjs';
import { summarize, toCsv } from './statistics.mjs';
import { captureNativeProvenance } from './native-provenance.mjs';
import { observeOwnedWindowsTree, terminateOwnedWindowsTree } from './process-tree.mjs';
import { captureSourceProvenance } from './source-provenance.mjs';
import { scaleProfile } from './scale-profiles.mjs';

const cwd = fileURLToPath(new URL('../', import.meta.url));
const cli = fileURLToPath(new URL('../../../packages/cli/bin/farm.mjs', import.meta.url));
const runId = `build-${Date.now()}-${randomUUID().slice(0, 8)}`;
const rounds = Number(process.env.COMMERCE_ROUNDS ?? 10);
const seed = Number(process.env.COMMERCE_SEED ?? 42);
const captureMemory = process.env.COMMERCE_CAPTURE_MEMORY === '1';
const scale = process.env.COMMERCE_SCALE ?? 'baseline';
scaleProfile(scale);
if (captureMemory && process.platform !== 'win32') throw new Error('Process-tree memory capture currently requires Windows');
const schedule = buildSchedule({ runId, rounds, seed });
const directory = path.join(cwd, 'reports', runId);
await fs.mkdir(directory, { recursive: true });
const report = { kind: 'production-build-samples', runId, seed, rounds, scale, captureMemory, publishable: false,
  limitations: ['Business workload and full capability acceptance remain incomplete.', captureMemory ? 'Instrumented Windows WorkingSetSize observations include collection overhead and may miss short-lived peaks.' : 'No process-tree memory sampling in this run.', 'Single-scale build run; no HMR measurement.'],
  environment: await captureEnvironment(), schedule, records: [] };
await fs.writeFile(path.join(directory, 'run.json'), JSON.stringify(report, null, 2));
const configs = { js: 'farm.config.ts', rust: 'farm.rust.config.ts', mixed: 'farm.mixed.config.ts' };
const successfulCold = new Set();
for (const sample of schedule) {
  const record = { ...sample, scenario: captureMemory ? 'production-build-instrumented' : 'production-build', scale, success: false, durationMs: null };
  report.records.push(record);
  const cachePath = path.join(cwd, '.cache', sample.group, sample.cacheId);
  const outputPath = path.join(cwd, 'dist', 'benchmark', sample.outputId, sample.group);
  if (sample.cacheMode === 'cold') {
    for (const target of [cachePath, outputPath]) {
      try { await fs.access(target); throw new Error(`Cold directory already exists: ${target}`); }
      catch (error) { if (error.code !== 'ENOENT') throw error; }
    }
  }
  if (sample.cacheMode === 'warm' && !successfulCold.has(sample.cacheId)) {
    record.error = 'Warm sample skipped: preceding cold build failed';
  } else {
    const logPath = path.join(directory, `${sample.round}-${sample.group}-${sample.cacheMode}.log`);
    const log = await fs.open(logPath, 'wx');
    const started = performance.now();
    const child = spawn(process.execPath, [cli, 'build', '--config', configs[sample.group]], {
      cwd, windowsHide: true, stdio: ['ignore', log.fd, log.fd],
      env: { ...process.env, COMMERCE_GROUP: sample.group, COMMERCE_CACHE_ID: sample.cacheId, COMMERCE_OUTPUT_ID: sample.outputId },
    });
    let timedOut = false;
    let termination = Promise.resolve(null);
    const memory = captureMemory ? observeOwnedWindowsTree(child) : null;
    const timer = setTimeout(() => {
      timedOut = true;
      termination = process.platform === 'win32'
        ? terminateOwnedWindowsTree(child).catch(error => { child.kill(); return { error: error.message, fallback: 'direct-child-only' }; })
        : Promise.resolve(child.kill());
    }, 600000);
    const outcome = await new Promise(resolve => {
      child.once('error', error => resolve({ error: error.message }));
      child.once('close', (exitCode, signal) => resolve({ exitCode, signal }));
    });
    clearTimeout(timer);
    record.durationMs = performance.now() - started;
    record.termination = await termination;
    if (memory) record.memory = await memory;
    await log.close();
    Object.assign(record, outcome, { timedOut, log: path.relative(cwd, logPath), outputPath, cachePath });
    if (!timedOut && outcome.exitCode === 0) {
      try {
        record.artifacts = await verifyCompression(outputPath);
        const worker = await fs.readFile(path.join(outputPath, 'import.js'), 'utf8');
        if (!worker.includes('postMessage')) throw new Error('Worker artifact lacks message handler');
        await fs.access(cachePath);
        record.success = true;
        if (sample.cacheMode === 'cold') successfulCold.add(sample.cacheId);
        if (captureMemory && !record.memory.valid) {
          record.success = false;
          record.error = 'No valid process-tree memory observation';
        }
      } catch (error) { record.error = error.message; }
    }
  }
  await fs.writeFile(path.join(directory, 'run.json'), JSON.stringify(report, null, 2));
  console.log(JSON.stringify({ round: sample.round, group: sample.group, cacheMode: sample.cacheMode, success: record.success, error: record.error }));
}
report.nativeAfterRun = await captureNativeProvenance();
report.sourceAfterRun = await captureSourceProvenance();
report.sourcesUnchanged = ['measurement', 'workload', 'configuration'].every(category => report.sourceAfterRun[category].sha256 === report.environment.source[category].sha256);
report.nativeArtifactsUnchanged = Object.entries(report.environment.native.artifacts).every(([name, before]) => {
  const after = report.nativeAfterRun.artifacts[name];
  return after?.path === before.path && after?.sha256 === before.sha256;
});
await fs.writeFile(path.join(directory, 'run.json'), JSON.stringify(report, null, 2));
await fs.writeFile(path.join(directory, 'raw.csv'), toCsv(report.records, ['round', 'group', 'scenario', 'scale', 'cacheMode', 'cacheId', 'outputId', 'success', 'durationMs', 'exitCode', 'signal', 'timedOut', 'error', 'log', 'artifacts', 'memory', 'termination']));
await fs.writeFile(path.join(directory, 'summary.json'), JSON.stringify(summarize(report.records), null, 2));
console.log(directory);
if (!report.sourcesUnchanged || !report.nativeArtifactsUnchanged || report.records.some(record => !record.success)) process.exitCode = 1;
