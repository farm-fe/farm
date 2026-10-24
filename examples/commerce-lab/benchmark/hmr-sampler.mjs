import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawn } from 'node:child_process';
import { randomUUID } from 'node:crypto';
import { buildSchedule } from './build-schedule.mjs';
import { captureEnvironment } from './environment.mjs';
import { captureNativeProvenance } from './native-provenance.mjs';
import { terminateOwnedWindowsTree, snapshotWindowsTree } from './process-tree.mjs';
import { waitForDevServer } from './dev-server-startup.mjs';
import { validateHmrRecords } from './hmr-records.mjs';
import { summarize, toCsv } from './statistics.mjs';
import { restoreSourceProbes } from './source-recovery.mjs';
import { captureSourceProvenance } from './source-provenance.mjs';
import { writeJsonCheckpoint } from './report-checkpoint.mjs';

const cwd = fileURLToPath(new URL('../', import.meta.url));
const cli = fileURLToPath(new URL('../../../packages/cli/bin/farm.mjs', import.meta.url));
const runId = `hmr-${Date.now()}-${randomUUID().slice(0, 8)}`;
const rounds = Number(process.env.COMMERCE_ROUNDS ?? 30);
const seed = Number(process.env.COMMERCE_SEED ?? 42);
const schedule = buildSchedule({ runId, rounds, seed }).filter(item => item.cacheMode === 'cold');
const directory = path.join(cwd, 'reports', runId);
await fs.mkdir(directory, { recursive: true });
const report = { kind: 'repeated-hmr-samples', runId, seed, rounds, publishable: false, schedule,
  protocol: 'One active dev server; restart per group/round; warm each scenario once, then measure six edits. Persistent cache retained per group. Browser observation includes file write and rendering, excludes server startup and warmup.',
  limitations: ['Current prototype workload; full release and capability acceptance incomplete.', 'Fresh browser pages and dev session per group/round; not a long-lived editing-session benchmark.', 'No process-tree memory in this timing run.'],
  environment: await captureEnvironment(), records: [], roundsCompleted: [] };
await writeJsonCheckpoint(path.join(directory, 'run.json'), report);
const configs = { js: 'farm.config.ts', rust: 'farm.rust.config.ts', mixed: 'farm.mixed.config.ts' };
async function stop(child) {
  if (!child || child.exitCode !== null || child.signalCode !== null) return;
  const closed = new Promise(resolve => child.once('close', resolve));
  if (process.platform === 'win32') await terminateOwnedWindowsTree(child);
  else child.kill();
  await closed;
}
async function driver(env, logName) {
  const edits = [
    ['apps/admin/pages/Components.tsx', 'Built to work together', 'Commerce leaf changed'],
    ['packages/ui/index.tsx', '<h1>{title}</h1>', '<h1>{title} shared changed</h1>'],
    ['apps/admin/styles/_tokens.scss', '#276d52', '#123456'],
    ['apps/admin/pages/Components.tsx', 'bg-emerald-100', 'bg-rose-200'],
    ['apps/admin/assets/brand.svg', 'rx="9"', 'rx="3"'],
    ['apps/admin/content/import-guide.mdx', 'Product import playbook', 'Commerce MDX changed'],
  ];
  const backups = new Map();
  for (const [file] of edits) if (!backups.has(file)) backups.set(file, await fs.readFile(path.join(cwd, file), 'utf8'));
  const log = await fs.open(path.join(directory, logName), 'wx');
  const child = spawn(process.execPath, ['tests/hmr-capabilities.mjs'], { cwd, env, windowsHide: true, stdio: ['ignore', log.fd, log.fd] });
  let timedOut = false;
  let terminating = Promise.resolve();
  const timer = setTimeout(() => { timedOut = true; terminating = stop(child); }, 600000);
  try {
    const code = await new Promise((resolve, reject) => { child.once('error', reject); child.once('close', resolve); });
    await terminating;
    return { code, timedOut };
  } finally {
    clearTimeout(timer); await log.close();
    const recovery = await restoreSourceProbes(cwd, backups, edits);
    await fs.writeFile(path.join(directory, `${logName}.recovery.json`), JSON.stringify(recovery, null, 2));
    if (recovery.conflicts.length || recovery.errors.length) throw new Error(`Source recovery failed: ${JSON.stringify(recovery)}`);
  }
}
let active;
try {
  for (const sample of schedule) {
    const port = 4340;
    const env = { ...process.env, COMMERCE_GROUP: sample.group, COMMERCE_PORT: String(port), COMMERCE_CACHE_ID: `${runId}-${sample.group}`, COMMERCE_OUTPUT_ID: `${runId}-${sample.group}`, COMMERCE_URL: `http://localhost:${port}` };
    const log = await fs.open(path.join(directory, `${sample.round}-${sample.group}-server.log`), 'wx');
    let logWrite = Promise.resolve();
    try {
      active = spawn(process.execPath, [cli, 'start', '--config', configs[sample.group]], { cwd, env, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] });
      const writeLog = chunk => {
        logWrite = logWrite.then(() => log.appendFile(chunk));
        logWrite.catch(() => {});
      };
      active.stdout.on('data', writeLog); active.stderr.on('data', writeLog);
      await waitForDevServer(active, {
        port,
        snapshot: snapshotWindowsTree,
        checkpoint: data => writeJsonCheckpoint(path.join(directory, `${sample.round}-${sample.group}-startup.json`), data),
      });
      for (const phase of ['warmup', 'measured']) {
        const filename = `${sample.round}-${sample.group}-${phase}.json`;
        const outcome = await driver({ ...env, COMMERCE_HMR_OUTPUT: path.join(directory, filename) }, `${sample.round}-${sample.group}-${phase}.log`);
        const raw = JSON.parse(await fs.readFile(path.join(directory, filename), 'utf8'));
        if (phase === 'measured') report.records.push(...raw.records.map(record => ({ ...record, round: sample.round, scale: 'current-prototype', cacheMode: 'dev-persistent', source: filename })));
        await writeJsonCheckpoint(path.join(directory, 'run.json'), report);
        validateHmrRecords(raw.records, sample.group);
        if (outcome.code !== 0 || outcome.timedOut) throw new Error('HMR driver failed');
      }
      report.roundsCompleted.push({ round: sample.round, group: sample.group });
      console.log(JSON.stringify({ round: sample.round, group: sample.group, success: true }));
    } finally { await stop(active); active = null; try { await logWrite; } finally { await log.close(); } }
  }
} catch (error) { report.error = error.message; process.exitCode = 1; }
finally {
  if (active) await stop(active);
  report.nativeAfterRun = await captureNativeProvenance();
  report.sourceAfterRun = await captureSourceProvenance();
  report.sourcesUnchanged = ['measurement', 'workload', 'configuration', 'workspaceRuntime'].every(category => report.sourceAfterRun[category].sha256 === report.environment.source[category].sha256);
  report.nativeArtifactsUnchanged = Object.entries(report.environment.native.artifacts).every(([name, before]) => report.nativeAfterRun.artifacts[name]?.sha256 === before.sha256 && report.nativeAfterRun.artifacts[name]?.path === before.path);
  report.wasmInputUnchanged = report.nativeAfterRun.wasmInput.sha256 === report.environment.native.wasmInput.sha256 && report.nativeAfterRun.wasmInput.path === report.environment.native.wasmInput.path;
  if (!report.sourcesUnchanged || !report.nativeArtifactsUnchanged || !report.wasmInputUnchanged) process.exitCode = 1;
  await writeJsonCheckpoint(path.join(directory, 'run.json'), report);
  await writeJsonCheckpoint(path.join(directory, 'summary.json'), summarize(report.records));
  await fs.writeFile(path.join(directory, 'raw.csv'), toCsv(report.records, ['round', 'group', 'scenario', 'scale', 'cacheMode', 'timestamp', 'success', 'durationMs', 'sameDocument', 'statePreserved', 'restored', 'error', 'restoreError', 'pageErrors', 'source']));
  console.log(directory);
}
