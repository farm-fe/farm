// cspell:words taskkill statfs bavail
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import { spawn, execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { performance } from 'node:perf_hooks';
import { gzipSync } from 'node:zlib';
import { quantile, toCsv } from '../examples/commerce-lab/benchmark/statistics.mjs';
import { extensionSources } from '../examples/commerce-lab/benchmark/extension-generator.mjs';
import { loadComparableHistory } from './performance-cohorts.mjs';
import { prepareComparisonFixture, COMPARISON_PROTOCOL, VITE_COMPARISON_VERSION } from '../examples/commerce-lab/benchmark/vite-comparison.mjs';
import { scaleProfile } from '../examples/commerce-lab/benchmark/scale-profiles.mjs';
import { resourceSnapshot, assertPerformanceResources } from './performance-resources.mjs';
import { diagnosticTail, redactDiagnostic } from './v2-release-diagnostics.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
export const protocol = 'commerce-business-2048-rust-build-v2';
export const workloadScale = 'business-2048';
export function businessFixture() { const profile = scaleProfile(workloadScale); return extensionSources(profile.categories, profile); }
export async function workloadFileCounts(sourceRoot, fixture) {
  let baseSourceFiles = 0;
  const lab = path.join(sourceRoot, 'examples/commerce-lab');
  async function visit(relative) {
    if (relative.replaceAll('\\', '/') === 'packages/extensions') return;
    const file = path.join(lab, relative), stat = await fs.lstat(file);
    if (stat.isSymbolicLink()) throw new Error('Workload input symlink is not counted');
    if (stat.isDirectory()) for (const name of await fs.readdir(file)) { if (!['node_modules', '.cache', 'dist', 'reports', 'extensions', '.git'].includes(name)) await visit(path.join(relative, name)); }
    else if (stat.isFile()) baseSourceFiles++;
  }
  await visit('');
  const generatedFiles = Object.keys(fixture).length;
  return { baseSourceFiles, generatedFiles, totalSourceFiles: baseSourceFiles + generatedFiles, scope: 'All copied workload inputs, including config, benchmark and test files', fileCountIsNotModuleCount: true };
}
export async function harnessFingerprint(sourceRoot) {
  const hash = crypto.createHash('sha256');
  for (const file of ['performance-report.mjs', 'performance-graph-worker.mjs', 'performance-resources.mjs', 'performance-cohorts.mjs']) hash.update(file).update((await fs.readFile(path.join(sourceRoot, 'scripts', file), 'utf8')).replaceAll('\r\n', '\n')).update('\0');
  return hash.digest('hex');
}
export function aggregate(records) {
  return ['cold', 'warm'].map(cacheMode => {
    const samples = records.filter(row => row.scored && row.cacheMode === cacheMode);
    const values = samples.filter(row => row.success).map(row => row.durationMs);
    return { cacheMode, samples: samples.length, validSamples: values.length,
      medianMs: quantile(values, .5), iqrMs: values.length ? quantile(values, .75) - quantile(values, .25) : null };
  });
}
export function compatible(a, b) {
  return ['protocol', 'fixtureSha256', 'harnessSha256', 'node', 'pnpm', 'os', 'arch', 'lockSha256'].every(key => a[key] !== undefined && a[key] === b[key]);
}
async function prepareLab(sourceRoot, temporary, fixture) {
  const lab = path.join(temporary, 'examples', 'commerce-lab');
  await fs.mkdir(path.dirname(lab), { recursive: true });
  await fs.cp(path.join(sourceRoot, 'examples/commerce-lab'), lab, { recursive: true, filter: source => !source.split(path.sep).some(part => ['node_modules', 'dist', '.cache', 'reports', 'extensions'].includes(part)) });
  await fs.copyFile(path.join(sourceRoot, 'tsconfig.base.json'), path.join(temporary, 'tsconfig.base.json'));
  for (const name of ['packages', 'rust-plugins', 'js-plugins', 'node_modules']) await fs.symlink(path.join(sourceRoot, name), path.join(temporary, name), process.platform === 'win32' ? 'junction' : 'dir');
  await fs.symlink(path.join(sourceRoot, 'examples/commerce-lab/node_modules'), path.join(lab, 'node_modules'), process.platform === 'win32' ? 'junction' : 'dir');
  for (const [name, content] of Object.entries(fixture)) {
    const file = path.join(lab, 'packages/extensions', workloadScale, name);
    await fs.mkdir(path.dirname(file), { recursive: true }); await fs.writeFile(file, content);
  }
  return lab;
}
export async function workloadFingerprint(sourceRoot, fixture) {
  const hash = crypto.createHash('sha256').update(JSON.stringify(fixture));
  const source = path.join(sourceRoot, 'examples/commerce-lab');
  async function visit(relative) {
    const target = path.join(source, relative);
    const stat = await fs.lstat(target);
    if (stat.isSymbolicLink()) throw new Error('Workload source may not contain symlinks');
    if (stat.isDirectory()) {
      for (const name of (await fs.readdir(target)).sort()) {
        const child = path.join(relative, name).replaceAll('\\', '/');
        // Hash every copied workload input, including HTML, adapters and scale
        // profiles. Only outputs/dependencies and regenerated extensions differ.
        if (['node_modules', 'dist', '.cache', 'reports', '.git'].includes(name) || child === 'packages/extensions') continue;
        await visit(child);
      }
    } else { hash.update(relative.replaceAll('\\', '/')).update('\0').update(await fs.readFile(target)).update('\0'); }
  }
  await visit('');
  return hash.digest('hex');
}
async function terminate(child) {
  if (process.platform === 'win32') {
    await new Promise((resolve, reject) => {
      const killer = spawn('taskkill', ['/pid', String(child.pid), '/T', '/F'], { windowsHide: true });
      let diagnostic = '';
      for (const stream of [killer.stdout, killer.stderr]) stream?.on('data', bytes => { diagnostic = (diagnostic + bytes.toString()).slice(-8192); });
      killer.once('error', reject); killer.once('close', code => code === 0 ? resolve() : reject(new Error(`taskkill failed: ${code}; ${diagnostic.trim()}`)));
    });
  } else {
    try { process.kill(-child.pid, 'SIGKILL'); } catch (error) { if (error.code !== 'ESRCH') throw error; }
  }
}
export async function waitForCommand(child, { timeoutMs = 120000, cleanupTimeoutMs = 5000, terminateProcess = terminate } = {}) {
  let timer;
  let expired = false;
  let onError, onClose;
  const exit = new Promise((resolve, reject) => {
    onError = reject; onClose = (code, signal) => resolve({ code, signal });
    child.once('error', onError); child.once('close', onClose);
  });
  const bounded = (promise, milliseconds) => {
    let deadline;
    return Promise.race([promise, new Promise((_, reject) => { deadline = setTimeout(() => reject(new Error('Cleanup deadline exceeded')), milliseconds); })]).finally(() => clearTimeout(deadline));
  };
  const timeout = new Promise((_, reject) => {
    timer = setTimeout(async () => {
      expired = true;
      let cleanupError;
      try { await bounded(terminateProcess(child), cleanupTimeoutMs); await bounded(exit, cleanupTimeoutMs); }
      catch (error) { cleanupError = error.message; }
      const error = new Error(`Command timedOut; pid=${child.pid}; ${cleanupError ? `cleanup failed: ${cleanupError}; process cleanup unconfirmed` : 'process tree terminated and handles closed'}`);
      Object.assign(error, { timedOut: true, pid: child.pid, cleanupConfirmed: !cleanupError, cleanupError });
      reject(error);
    }, timeoutMs);
  });
  try { return await Promise.race([exit.then(result => expired ? timeout : result), timeout]); }
  finally { clearTimeout(timer); child.removeListener('error', onError); child.removeListener('close', onClose); }
}
export async function command(executable, args, { cwd, env, log, timeoutMs = 120000, phase = 'command' } = {}) {
  const started = performance.now();
  const identity = { phase, cwd, timeoutMs };
  console.log('[command start] ' + JSON.stringify(identity));
  const handle = await fs.open(log, 'a');
  try {
    const child = spawn(executable, args, { cwd, env, windowsHide: true, detached: process.platform !== 'win32', stdio: ['ignore', handle.fd, handle.fd] });
    const exit = await waitForCommand(child, { timeoutMs });
    if (exit.code !== 0) throw Object.assign(new Error(`Command failed: ${executable} ${args.join(' ')} (${JSON.stringify(exit)})`), { exitCode: exit.code, signal: exit.signal, pid: child.pid });
  } catch (error) {
    Object.assign(error, { phase, cwd, timeoutMs, elapsedMs: performance.now() - started });
    try {
      console.error('[command failed] ' + JSON.stringify({ ...identity, elapsedMs: error.elapsedMs,
        error: redactDiagnostic(error.message, env ?? process.env), exitCode: error.exitCode ?? null,
        timedOut: error.timedOut === true, cleanupConfirmed: error.cleanupConfirmed ?? null,
        tail: diagnosticTail(log, env ?? process.env) }));
    } catch (diagnosticError) { console.error('[command diagnostics unavailable] ' + redactDiagnostic(diagnosticError.message, env ?? process.env)); }
    throw error;
  } finally { await handle.close(); }
}
export function rethrowUnsafeCleanup(error) {
  if (error.cleanupConfirmed === false) throw error;
}
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
async function sizes(directory) {
  const totals = { js: { bytes: 0, gzipBytes: 0 }, css: { bytes: 0, gzipBytes: 0 } };
  for (const entry of await fs.readdir(directory, { withFileTypes: true })) {
    const file = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      const child = await sizes(file);
      for (const type of ['js', 'css']) for (const key of ['bytes', 'gzipBytes']) totals[type][key] += child[type][key];
    } else {
      const type = /\.(js|css)$/.exec(entry.name)?.[1];
      if (type) { const bytes = await fs.readFile(file); totals[type].bytes += bytes.length; totals[type].gzipBytes += gzipSync(bytes).length; }
    }
  }
  return totals;
}
async function trackedFingerprint() {
  const names = execFileSync('git', ['ls-files', '-z', 'examples/commerce-lab'], { cwd: root, encoding: 'utf8' }).split('\0').filter(Boolean).sort();
  const hasher = crypto.createHash('sha256');
  for (const name of names) { hasher.update(name); hasher.update(await fs.readFile(path.join(root, name))); }
  return hasher.digest('hex');
}
async function main() {
  const output = path.resolve(process.env.FARM_PERFORMANCE_OUTPUT ?? path.join(root, 'performance-results'));
  await fs.mkdir(output, { recursive: true });
  const report = { schemaVersion: 1, protocol, status: 'partial', publishable: false, records: [], errors: [], comparison: { mode: 'current-only', reason: 'No compatible exact-base-SHA native cohort and workload receipt supplied; no regression conclusion.' } };
  let temp;
  let before;
  let phase = 'identity';
  const startedAt = Number(process.env.FARM_PERFORMANCE_STARTED_AT ?? Date.now());
  if (!Number.isFinite(startedAt) || startedAt <= 0 || startedAt > Date.now()) throw new Error('Invalid performance job start timestamp');
  const deadline = startedAt + 18 * 60 * 1000;
  const log = path.join(output, 'run.log');
  const persist = async () => fs.writeFile(path.join(output, 'raw.json'), `${JSON.stringify(report, null, 2)}\n`);
  await persist();
  try {
    const revision = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim();
    report.metadata = { protocol, revision, artifactProducerSHA: process.env.GITHUB_SHA ?? revision, baseSHA: process.env.FARM_BASE_SHA ?? null,
      os: process.platform, arch: process.arch, node: process.version, runnerImage: process.env.ImageOS ?? process.env.RUNNER_OS ?? process.platform, runnerImageVersion: process.env.ImageVersion ?? null,
      pnpm: null, fixtureSha256: null, harnessSha256: null, lockSha256: null,
      scale: { name: workloadScale, ...scaleProfile(workloadScale), fixtureGenerated: false } };
    await persist();
    if (process.env.GITHUB_SHA && revision !== process.env.GITHUB_SHA) throw new Error('Checkout does not match artifact producer SHA');
    phase = 'resources';
    const disk = await fs.statfs(os.tmpdir());
    report.resources = resourceSnapshot({ diskFreeBytes: disk.bavail * disk.bsize }); await persist();
    assertPerformanceResources(report.resources);
    phase = 'fixture';
    const fixture = businessFixture();
    report.metadata = { protocol, revision, baseSHA: process.env.FARM_BASE_SHA ?? null,
      os: process.platform, arch: process.arch, cpu: os.cpus()[0]?.model, runner: process.env.RUNNER_NAME,
      runnerImage: process.env.ImageOS ?? process.env.RUNNER_OS ?? process.platform, runnerImageVersion: process.env.ImageVersion,
      node: process.version, pnpm: execFileSync(process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm', ['--version'], { encoding: 'utf8', shell: process.platform === 'win32' }).trim(),
      lockSha256: hash(await fs.readFile(path.join(root, 'pnpm-lock.yaml'))), fixtureSha256: await workloadFingerprint(root, fixture), harnessSha256: await harnessFingerprint(root),
      scale: { name: workloadScale, ...scaleProfile(workloadScale), fixtureGenerated: true, generatedFileCount: Object.keys(fixture).length, fileCountIsNotModuleCount: true },
      cacheSemantics: 'Unique cold directory; warm reuses successful cold directory. Reusable cache, not independently proven cache hit.',
      artifactProducerSHA: process.env.GITHUB_SHA ?? revision };
    report.metadata.scale.files = await workloadFileCounts(root, fixture);
    before = await trackedFingerprint();
    const { captureNativeProvenance } = await import('../examples/commerce-lab/benchmark/native-provenance.mjs');
    report.native = await captureNativeProvenance({ groups: ['rust'] });
    temp = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-perf-'));
    const lab = await prepareLab(root, temp, fixture);
    phase = 'sampling';
    const cohorts = [{ root, lab, records: report.records, name: 'head' }];
    try {
      const base = JSON.parse(await fs.readFile(path.join(root, '.performance-base-receipt.json'), 'utf8'));
      if (base.mode !== 'paired') throw new Error(base.reason);
      const baseRoot = path.join(root, '.performance-base');
      if (path.resolve(base.root) !== baseRoot || base.revision !== report.metadata.baseSHA) throw new Error('Base receipt identity mismatch');
      const baseMetadata = { ...report.metadata, revision: base.revision,
        fixtureSha256: await workloadFingerprint(baseRoot, fixture), harnessSha256: await harnessFingerprint(baseRoot), lockSha256: hash(await fs.readFile(path.join(baseRoot, 'pnpm-lock.yaml'))) };
      if (!compatible(report.metadata, baseMetadata)) throw new Error('Base workload, lockfile or protocol is incompatible');
      const baseTemp = path.join(temp, 'base'); await fs.mkdir(baseTemp);
      report.comparison = { mode: 'paired', reason: 'Same runner, fixture, lockfile and toolchain; isolated exact-base native cohort', metadata: baseMetadata, records: [], native: base.nativeProvenance };
      cohorts.push({ root: baseRoot, lab: await prepareLab(baseRoot, baseTemp, fixture), records: report.comparison.records, name: 'base' });
    } catch (error) { report.comparison.reason = `Current only: ${error.message}`; }
    await persist();
    for (let round = -1; round < 3; round++) {
      const id = `perf-${round < 0 ? 'warmup' : round}`;
      const coldSuccess = new Set();
      for (const cacheMode of round < 0 ? ['cold'] : ['cold', 'warm']) {
       for (const cohort of round % 2 === 0 ? cohorts : [...cohorts].reverse()) {
        if (Date.now() >= deadline) throw new Error('Performance protocol deadline reached');
        const disk = await fs.statfs(temp);
        report.resources = resourceSnapshot({ diskFreeBytes: disk.bavail * disk.bsize }); assertPerformanceResources(report.resources);
        const record = { round, scored: round >= 0, cacheMode, cohort: cohort.name, success: false, durationMs: null };
        cohort.records.push(record);
        if (cacheMode === 'warm' && !coldSuccess.has(cohort.name)) { record.error = 'Preceding cold build failed'; await persist(); continue; }
        const started = performance.now();
        try {
          await command(process.execPath, [path.join(cohort.root, 'packages/cli/bin/farm.mjs'), 'build', '--config', 'farm.rust.config.ts'], { phase: `${cohort.name}/${round < 0 ? 'warmup' : 'sample-' + round}/${cacheMode}`, cwd: cohort.lab, log, timeoutMs: Math.min(120000, deadline - Date.now()), env: { ...process.env, COMMERCE_SCALE: workloadScale, COMMERCE_GROUP: 'rust', COMMERCE_CACHE_ID: id, COMMERCE_OUTPUT_ID: id } });
          record.durationMs = performance.now() - started;
          record.artifacts = await sizes(path.join(cohort.lab, 'dist/benchmark', id, 'rust'));
          if (record.artifacts.js.bytes === 0) throw new Error('Build produced no JavaScript');
          await fs.access(path.join(cohort.lab, '.cache/rust', id));
          record.success = true; if (cacheMode === 'cold') coldSuccess.add(cohort.name);
        } catch (error) { record.error = error.message; rethrowUnsafeCleanup(error); }
        await persist();
       }
      }
    }
    report.comparison.baseCells = aggregate(report.comparison.records ?? []);
    report.graphs = [];
    for (const cohort of cohorts) {
      const graphDisk = await fs.statfs(temp);
      report.resources = resourceSnapshot({ diskFreeBytes: graphDisk.bavail * graphDisk.bsize }); assertPerformanceResources(report.resources);
      const graphFile = path.join(temp, `graph-${cohort.name}.json`);
      await command(process.execPath, [path.join(cohort.root, 'scripts/performance-graph-worker.mjs'), cohort.lab, graphFile], { phase: `${cohort.name}/graph`, cwd: cohort.lab, log,
        timeoutMs: Math.min(120000, deadline - Date.now()), env: { ...process.env, COMMERCE_SCALE: workloadScale, COMMERCE_GROUP: 'rust', COMMERCE_CACHE_ID: 'graph-final', COMMERCE_OUTPUT_ID: 'graph-final' } });
      const graph = JSON.parse(await fs.readFile(graphFile, 'utf8'));
      if (graph.status !== 'complete' || !Number.isInteger(graph.summary?.retainedModules) || graph.summary.retainedModules <= 0) throw new Error('Actual graph count is unavailable');
      report.graphs.push({ cohort: cohort.name, ...graph });
    }
    const comparisonRoot = path.join(temp, 'vite-subset');
    const subset = await prepareComparisonFixture(comparisonRoot);
    report.vite = { protocol: COMPARISON_PROTOCOL, version: VITE_COMPARISON_VERSION, fingerprint: subset.fingerprint, records: [], publishable: false,
      semantics: 'Identical catalog and quote source; isolated production build API timing, persistent cache disabled for both. Not the full Rust Commerce workload.' };
    for (let round = -1; round < 3; round++) for (const tool of round % 2 === 0 ? ['farm', 'vite'] : ['vite', 'farm']) {
      if (Date.now() >= deadline) throw new Error('Deadline reached before comparison sample');
      const disk = await fs.statfs(temp);
      report.resources = resourceSnapshot({ diskFreeBytes: disk.bavail * disk.bsize }); assertPerformanceResources(report.resources);
      const record = { round, tool, scored: round >= 0, cacheMode: 'cold', success: false, durationMs: null };
      report.vite.records.push(record);
      const result = path.join(comparisonRoot, `${tool}-${round}.json`);
      try {
        await command(process.execPath, [path.join(root, 'scripts/performance-vite-worker.mjs'), tool, comparisonRoot, result], { cwd: root, env: process.env, log, timeoutMs: Math.min(120000, deadline - Date.now()) });
        const sample = JSON.parse(await fs.readFile(result, 'utf8'));
        if (sample.protocol !== COMPARISON_PROTOCOL || sample.viteVersion !== VITE_COMPARISON_VERSION || !Number.isFinite(sample.buildMs) || sample.buildMs < 0) throw new Error('Invalid comparison sample');
        Object.assign(record, { success: true, durationMs: sample.buildMs, artifacts: sample.artifacts });
      } catch (error) { record.error = error.message; rethrowUnsafeCleanup(error); }
      await persist();
    }
    report.vite.farmCells = aggregate(report.vite.records.filter(row => row.tool === 'farm')).filter(row => row.cacheMode === 'cold');
    report.vite.viteCells = aggregate(report.vite.records.filter(row => row.tool === 'vite')).filter(row => row.cacheMode === 'cold');
    report.vite.status = report.vite.records.every(row => row.success) ? 'complete' : 'invalid';
    // Functional equivalence is checked after all timed samples, never charged
    // to either builder. The existing owned-process timeout also covers Chromium.
    if (report.vite.status === 'complete') {
      const correctnessFile = path.join(comparisonRoot, 'correctness.json');
      await command(process.execPath, [path.join(root, 'scripts/performance-correctness-worker.mjs'), comparisonRoot, correctnessFile], { cwd: root, env: process.env, log, timeoutMs: Math.min(90000, deadline - Date.now()) });
      report.vite.correctness = JSON.parse(await fs.readFile(correctnessFile, 'utf8'));
      if (report.vite.correctness.protocol !== COMPARISON_PROTOCOL || report.vite.correctness.status !== 'complete' || !['farm', 'vite'].every(tool => report.vite.correctness.results?.[tool]?.functional === true)) throw new Error('Comparison browser correctness failed');
    }
    report.history = await loadComparableHistory({ repository: process.env.GITHUB_REPOSITORY, abi: process.env.FARM_PERFORMANCE_ABI, metadata: report.metadata });
    report.status = report.records.every(row => row.success) && (report.comparison.records ?? []).every(row => row.success) && report.vite.status === 'complete' ? 'complete' : 'invalid';
    const nativeAfter = await captureNativeProvenance({ groups: ['rust'] });
    report.nativeUnchanged = Object.entries(report.native.artifacts).every(([name, artifact]) => nativeAfter.artifacts[name]?.sha256 === artifact.sha256);
    if (!report.nativeUnchanged) throw new Error('Native cohort changed during sampling');
  } catch (error) { report.status = 'invalid'; report.cleanupUnconfirmed = error.cleanupConfirmed === false; report.failureKind = report.cleanupUnconfirmed ? 'cleanup-failure' : error.failureKind ?? (phase === 'identity' ? 'identity-failure' : phase === 'resources' ? 'resource-unavailable' : 'sampling-failure'); report.errors.push(error.message); }
  finally {
    if (before) { report.sourcesUnchanged = before === await trackedFingerprint(); if (!report.sourcesUnchanged) { report.status = 'invalid'; report.errors.push('Tracked Commerce source changed'); } }
    if (temp && report.cleanupUnconfirmed) { report.temporaryDirectoryRemoved = false; report.errors.push(`Temporary inputs retained because process cleanup is unconfirmed: ${temp}`); }
    else if (temp) { try { await fs.rm(temp, { recursive: true }); report.temporaryDirectoryRemoved = true; } catch (error) { report.status = 'invalid'; report.errors.push(`Cleanup: ${error.message}`); } }
    report.cells = aggregate(report.records);
    report.limitations = ['Three scored pairs are insufficient for formal performance certification.', 'No HMR, RSS or CPU sampling.', 'No cross-OS absolute ranking.', report.comparison.reason];
    await persist();
    await fs.writeFile(path.join(output, 'summary.json'), JSON.stringify({ ...report, records: undefined, native: undefined }, null, 2));
    await fs.writeFile(path.join(output, 'samples.csv'), toCsv([...report.records, ...(report.comparison.records ?? []), ...(report.vite?.records ?? []).map(row => ({ ...row, cohort: 'catalog-quote-subset' }))], ['round', 'scored', 'cohort', 'tool', 'cacheMode', 'success', 'durationMs', 'error', 'artifacts']));
    const markdown = `### Commerce performance (${process.platform}/${process.arch})\n\nStatus: **${report.status}**; publishable: **false**; comparison: **${report.comparison.mode}**.\n\n| Cache | Valid samples | Median ms | IQR ms |\n|---|---:|---:|---:|\n${report.cells.map(cell => `| ${cell.cacheMode} | ${cell.validSamples} | ${cell.medianMs ?? 'unavailable'} | ${cell.iqrMs ?? 'unavailable'} |`).join('\n')}\n\n${report.errors.join('\n')}\n\n${report.limitations.join('\n\n')}\n`;
    await fs.writeFile(path.join(output, 'summary.md'), markdown);
    if (process.env.GITHUB_STEP_SUMMARY) await fs.appendFile(process.env.GITHUB_STEP_SUMMARY, markdown);
    if (report.status !== 'complete') process.exitCode = 1;
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
