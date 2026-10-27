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

const root = fileURLToPath(new URL('../', import.meta.url));
export const protocol = 'commerce-baseline-rust-build-v1';
export function aggregate(records) {
  return ['cold', 'warm'].map(cacheMode => {
    const samples = records.filter(row => row.scored && row.cacheMode === cacheMode);
    const values = samples.filter(row => row.success).map(row => row.durationMs);
    return { cacheMode, samples: samples.length, validSamples: values.length,
      medianMs: quantile(values, .5), iqrMs: values.length ? quantile(values, .75) - quantile(values, .25) : null };
  });
}
export function compatible(a, b) {
  return ['protocol', 'fixtureSha256', 'node', 'pnpm', 'os', 'arch', 'lockSha256'].every(key => a[key] !== undefined && a[key] === b[key]);
}
async function terminate(child) {
  if (process.platform === 'win32') {
    await new Promise((resolve, reject) => {
      const killer = spawn('taskkill', ['/pid', String(child.pid), '/T', '/F'], { windowsHide: true });
      killer.once('error', reject); killer.once('close', code => code === 0 ? resolve() : reject(new Error(`taskkill failed: ${code}`)));
    });
  } else {
    try { process.kill(-child.pid, 'SIGKILL'); } catch (error) { if (error.code !== 'ESRCH') throw error; }
  }
}
export async function command(executable, args, { cwd, env, log, timeoutMs = 120000 } = {}) {
  const handle = await fs.open(log, 'a');
  let timer;
  let timedOut = false;
  let cleanup = Promise.resolve();
  try {
    const child = spawn(executable, args, { cwd, env, windowsHide: true, detached: process.platform !== 'win32', stdio: ['ignore', handle.fd, handle.fd] });
    timer = setTimeout(() => { timedOut = true; cleanup = terminate(child); cleanup.catch(() => {}); }, timeoutMs);
    const exit = await new Promise((resolve, reject) => { child.once('error', reject); child.once('close', (code, signal) => resolve({ code, signal })); });
    await cleanup;
    if (timedOut || exit.code !== 0) throw new Error(`Command failed: ${executable} ${args.join(' ')} (${JSON.stringify({ ...exit, timedOut })})`);
  } finally { clearTimeout(timer); await handle.close(); }
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
  const deadline = Date.now() + 18 * 60 * 1000;
  const log = path.join(output, 'run.log');
  const persist = async () => fs.writeFile(path.join(output, 'raw.json'), `${JSON.stringify(report, null, 2)}\n`);
  await persist();
  try {
    const disk = await fs.statfs(os.tmpdir());
    if (os.freemem() < 3.4 * 1024 ** 3 || disk.bavail * disk.bsize < 8 * 1024 ** 3) throw new Error('Resource guard: requires 3.4 GiB available RAM and 8 GiB free scratch disk');
    const revision = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim();
    if (process.env.GITHUB_SHA && revision !== process.env.GITHUB_SHA) throw new Error('Checkout does not match artifact producer SHA');
    const fixture = extensionSources(4);
    report.metadata = { protocol, revision, baseSHA: process.env.FARM_BASE_SHA ?? null,
      os: process.platform, arch: process.arch, cpu: os.cpus()[0]?.model, runner: process.env.RUNNER_NAME,
      runnerImage: process.env.ImageOS ?? process.env.RUNNER_OS, runnerImageVersion: process.env.ImageVersion,
      node: process.version, pnpm: execFileSync(process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm', ['--version'], { encoding: 'utf8', shell: process.platform === 'win32' }).trim(),
      lockSha256: hash(await fs.readFile(path.join(root, 'pnpm-lock.yaml'))), fixtureSha256: hash(JSON.stringify(fixture)), scale: { name: 'baseline', categories: 4 },
      cacheSemantics: 'Unique cold directory; warm reuses successful cold directory. Reusable cache, not independently proven cache hit.',
      artifactProducerSHA: process.env.GITHUB_SHA ?? revision };
    before = await trackedFingerprint();
    const { captureNativeProvenance } = await import('../examples/commerce-lab/benchmark/native-provenance.mjs');
    report.native = await captureNativeProvenance({ groups: ['rust'] });
    temp = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-perf-'));
    const lab = path.join(temp, 'examples', 'commerce-lab');
    await fs.mkdir(path.dirname(lab), { recursive: true });
    await fs.cp(path.join(root, 'examples/commerce-lab'), lab, { recursive: true, filter: source => !source.split(path.sep).some(part => ['node_modules', 'dist', '.cache', 'reports', 'extensions'].includes(part)) });
    for (const name of ['packages', 'rust-plugins', 'js-plugins', 'node_modules']) await fs.symlink(path.join(root, name), path.join(temp, name), process.platform === 'win32' ? 'junction' : 'dir');
    await fs.symlink(path.join(root, 'examples/commerce-lab/node_modules'), path.join(lab, 'node_modules'), process.platform === 'win32' ? 'junction' : 'dir');
    for (const [name, content] of Object.entries(fixture)) {
      const file = path.join(lab, 'packages/extensions/baseline', name);
      await fs.mkdir(path.dirname(file), { recursive: true }); await fs.writeFile(file, content);
    }
    await persist();
    for (let round = -1; round < 3; round++) {
      const id = `perf-${round < 0 ? 'warmup' : round}`;
      let coldSuccess = false;
      for (const cacheMode of round < 0 ? ['cold'] : ['cold', 'warm']) {
        if (Date.now() >= deadline) throw new Error('Performance protocol deadline reached');
        const disk = await fs.statfs(temp);
        if (os.freemem() < 3.4 * 1024 ** 3 || disk.bavail * disk.bsize < 8 * 1024 ** 3) throw new Error('Resource guard stopped sampling before the next build');
        const record = { round, scored: round >= 0, cacheMode, success: false, durationMs: null };
        report.records.push(record);
        if (cacheMode === 'warm' && !coldSuccess) { record.error = 'Preceding cold build failed'; await persist(); continue; }
        const started = performance.now();
        try {
          await command(process.execPath, [path.join(root, 'packages/cli/bin/farm.mjs'), 'build', '--config', 'farm.rust.config.ts'], { cwd: lab, log, timeoutMs: Math.min(120000, deadline - Date.now()), env: { ...process.env, COMMERCE_SCALE: 'baseline', COMMERCE_GROUP: 'rust', COMMERCE_CACHE_ID: id, COMMERCE_OUTPUT_ID: id } });
          record.durationMs = performance.now() - started;
          record.artifacts = await sizes(path.join(lab, 'dist/benchmark', id, 'rust'));
          if (record.artifacts.js.bytes === 0) throw new Error('Build produced no JavaScript');
          await fs.access(path.join(lab, '.cache/rust', id));
          record.success = true; if (cacheMode === 'cold') coldSuccess = true;
        } catch (error) { record.error = error.message; }
        await persist();
      }
    }
    report.status = report.records.every(row => row.success) ? 'complete' : 'invalid';
    const nativeAfter = await captureNativeProvenance({ groups: ['rust'] });
    report.nativeUnchanged = Object.entries(report.native.artifacts).every(([name, artifact]) => nativeAfter.artifacts[name]?.sha256 === artifact.sha256);
    if (!report.nativeUnchanged) throw new Error('Native cohort changed during sampling');
  } catch (error) { report.status = 'invalid'; report.errors.push(error.message); }
  finally {
    if (before) { report.sourcesUnchanged = before === await trackedFingerprint(); if (!report.sourcesUnchanged) { report.status = 'invalid'; report.errors.push('Tracked Commerce source changed'); } }
    if (temp) { try { await fs.rm(temp, { recursive: true }); report.temporaryDirectoryRemoved = true; } catch (error) { report.status = 'invalid'; report.errors.push(`Cleanup: ${error.message}`); } }
    report.cells = aggregate(report.records);
    report.limitations = ['Three scored pairs are insufficient for formal performance certification.', 'No HMR, RSS or CPU sampling.', 'No cross-OS absolute ranking.', report.comparison.reason];
    await persist();
    await fs.writeFile(path.join(output, 'summary.json'), JSON.stringify({ ...report, records: undefined, native: undefined }, null, 2));
    await fs.writeFile(path.join(output, 'samples.csv'), toCsv(report.records, ['round', 'scored', 'cacheMode', 'success', 'durationMs', 'error', 'artifacts']));
    const markdown = `### Commerce performance (${process.platform}/${process.arch})\n\nStatus: **${report.status}**; publishable: **false**; comparison: **current-only**.\n\n| Cache | Valid samples | Median ms | IQR ms |\n|---|---:|---:|---:|\n${report.cells.map(cell => `| ${cell.cacheMode} | ${cell.validSamples} | ${cell.medianMs ?? 'unavailable'} | ${cell.iqrMs ?? 'unavailable'} |`).join('\n')}\n\n${report.errors.join('\n')}\n\n${report.limitations.join('\n\n')}\n`;
    await fs.writeFile(path.join(output, 'summary.md'), markdown);
    if (process.env.GITHUB_STEP_SUMMARY) await fs.appendFile(process.env.GITHUB_STEP_SUMMARY, markdown);
    if (report.status !== 'complete') process.exitCode = 1;
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
