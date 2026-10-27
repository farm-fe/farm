// cspell:words statfs bavail taskkill
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
export function inspectSdkMetadata(metadata, materialization, lock) {
  assert.equal(metadata.workspace_members.length, 1, 'SDK consumer must have its own workspace');
  const expected = [...materialization.selected, ...materialization.reused];
  assert.equal(new Set(expected.map(row => row.name)).size, expected.length);
  assert.ok(!metadata.packages.some(row => ['wax', 'tardar'].includes(row.name)), 'Removed dependency reentered standalone SDK');
  const packages = [];
  for (const record of expected) {
    const rows = metadata.packages.filter(row => row.name === record.name && row.version === record.version);
    assert.equal(rows.length, 1, `Missing or duplicate SDK package: ${record.name}`);
    assert.equal(rows[0].version, record.version);
    assert.match(rows[0].source ?? '', /^registry\+/, 'SDK package came from a workspace/path');
    assert.ok(!metadata.workspace_members.includes(rows[0].id));
    const entry = lock.split('[[package]]').find(block => block.includes(`name = "${record.name}"\n`) && block.includes(`version = "${record.version}"\n`));
    assert.ok(entry?.includes(`checksum = "${record.sha256}"`), `Locked SDK bytes differ: ${record.name}`);
    const otherVersions = metadata.packages.filter(row => row.name === record.name && row.version !== record.version);
    const compatibility = [];
    for (const extra of otherVersions) {
      // enhanced-magic-string's private historical utility dependency is not a
      // replacement for the exact V2 utility used by Farm's own SDK packages.
      assert.equal(extra.name, 'farmfe_utils', 'Unreviewed additional SDK crate version');
      assert.equal(extra.version, '0.1.6', 'Unreviewed historical utility version');
      assert.match(extra.source ?? '', /^registry\+/);
      const parents = metadata.resolve.nodes.filter(node => node.deps.some(dep => dep.pkg === extra.id)).map(node => metadata.packages.find(pkg => pkg.id === node.id));
      assert.ok(parents.length && parents.every(pkg => pkg.name === 'enhanced-magic-string' && pkg.version === '0.1.0'), 'Farm SDK must not directly bind the historical utility');
      compatibility.push({ name: extra.name, version: extra.version, source: extra.source, requiredBy: parents.map(pkg => `${pkg.name}@${pkg.version}`) });
    }
    packages.push({ name: record.name, version: record.version, sha256: record.sha256, source: rows[0].source, ...(compatibility.length ? { compatibilityTransitives: compatibility } : {}) });
  }
  return packages;
}
async function main() {
  const output = path.resolve(process.argv[2] ?? '');
  if (!process.argv[2]) throw new Error('Expected prepared SDK evidence directory');
  const materialization = JSON.parse(fs.readFileSync(path.join(output, 'materialization.json')));
  const plan = JSON.parse(fs.readFileSync(fileURLToPath(new URL('../docs/releases/v2-candidate.json', import.meta.url))));
  assert.equal(materialization.planSha256, hash(JSON.stringify(plan)), 'SDK registry belongs to a different frozen release plan');
  assert.deepEqual([...materialization.selected, ...materialization.reused].map(({ name, version }) => `${name}@${version}`).sort(), plan.cargo.map(({ name, version }) => `${name}@${version}`).sort(), 'SDK candidate package coverage mismatch');
  const consumer = path.join(output, 'consumer');
  assert.equal(path.resolve(materialization.consumer), consumer);
  assert.equal(materialization.workspacePatches, false);
  const manifest = fs.readFileSync(path.join(consumer, 'Cargo.toml'), 'utf8');
  assert.doesNotMatch(manifest, /^\s*\[patch|\bpath\s*=/m, 'Workspace patch/path dependency in consumer');
  const log = path.join(output, 'sdk-test.log');
  if (fs.existsSync(log)) throw new Error('Inspect previous SDK command outcome before retrying');
  fs.writeFileSync(log, '');
  const reportFile = path.join(output, 'sdk-consumer.json');
  const report = { schemaVersion: 1, status: 'running', phase: 'start', sourceMode: 'sealed-local-registry',
    sourceCommit: materialization.sourceCommit, planSha256: materialization.planSha256,
    workspacePatches: false, published: false, packages: [], log: 'sdk-test.log', startedAt: new Date().toISOString() };
  const save = () => fs.writeFileSync(reportFile, JSON.stringify(report, null, 2));
  const cargo = process.env.FARM_CARGO_EXECUTABLE ?? (process.platform === 'win32' ? path.join(os.homedir(), '.cargo/bin/cargo.exe') : 'cargo');
  const target = process.env.FARM_SDK_TARGET_DIR ?? path.join(output, 'target');
  const environment = { ...process.env, CARGO_HOME: process.env.FARM_SDK_CARGO_HOME ?? path.join(output, 'cargo-home'), CARGO_TARGET_DIR: target, CARGO_NET_OFFLINE: 'true' };
  const windowEnd = process.env.FARM_SDK_WINDOW_END ? Date.parse(process.env.FARM_SDK_WINDOW_END) : Date.now() + 120 * 60 * 1000;
  assert.ok(Number.isFinite(windowEnd) && windowEnd > Date.now(), 'SDK allocated window has expired');
  const deadline = Math.min(windowEnd, Date.now() + 120 * 60 * 1000);
  report.windowEndsAt = new Date(deadline).toISOString();
  async function command(phase, args, metadataOutput) {
    const disk = fs.statfsSync(output);
    report.resources = { freeRamGiB: os.freemem() / 1024 ** 3, freeDiskGiB: disk.bavail * disk.bsize / 1024 ** 3 };
    if (report.resources.freeRamGiB < 3.4 || report.resources.freeDiskGiB < 9.75) throw new Error('SDK command resource guard');
    report.phase = phase; save();
    const fd = fs.openSync(log, 'a');
    const stdout = metadataOutput ? fs.openSync(metadataOutput, 'w') : fd;
    fs.writeSync(fd, `\nPHASE ${phase}: cargo ${args.join(' ')}\n`);
    const child = spawn(cargo, ['+nightly-2026-04-10', ...args], { cwd: consumer, env: environment, windowsHide: true,
      detached: process.platform !== 'win32', stdio: ['ignore', stdout, fd] });
    report.pid = child.pid; save(); console.log(JSON.stringify({ phase, pid: child.pid, resources: report.resources }));
    let reason, kill = Promise.resolve();
    const stop = message => {
      if (reason) return;
      reason = message;
      if (process.platform === 'win32') kill = new Promise((resolve, reject) => {
        const killer = spawn('taskkill', ['/pid', String(child.pid), '/T', '/F'], { windowsHide: true });
        killer.once('error', reject); killer.once('close', code => code === 0 ? resolve() : reject(new Error(`Owned SDK tree cleanup failed: ${code}`)));
      });
      else { try { process.kill(-child.pid, 'SIGKILL'); } catch (error) { if (error.code !== 'ESRCH') throw error; } }
      kill.catch(() => {});
    };
    const timer = setInterval(() => {
      try {
        if (Date.now() >= deadline) stop('SDK 120-minute command budget exceeded');
        const state = fs.statfsSync(output);
        if (state.bavail * state.bsize < 8 * 1024 ** 3) stop('SDK absolute disk guard');
      } catch { stop('SDK resource probe failed'); }
    }, 5000);
    let result;
    try { result = await new Promise((resolve, reject) => { child.once('error', reject); child.once('close', code => resolve(code)); }); await kill; }
    finally { clearInterval(timer); fs.closeSync(fd); if (metadataOutput) fs.closeSync(stdout); delete report.pid; }
    assert.equal(result, 0, reason ?? `SDK ${phase} failed`);
  }
  try {
    save();
    await command('lock', ['generate-lockfile', '--offline']);
    await command('metadata', ['metadata', '--format-version', '1', '--offline', '--locked'], path.join(output, 'sdk-metadata.json'));
    const lock = fs.readFileSync(path.join(consumer, 'Cargo.lock'), 'utf8').replaceAll('\r\n', '\n');
    report.packages = inspectSdkMetadata(JSON.parse(fs.readFileSync(path.join(output, 'sdk-metadata.json'))), materialization, lock);
    await command('tests', ['test', '--release', '--offline', '--locked', '--jobs', '1']);
    assert.match(fs.readFileSync(log, 'utf8'), /test result: ok\. 3 passed; 0 failed/);
    fs.copyFileSync(path.join(consumer, 'Cargo.toml'), path.join(output, 'consumer.Cargo.toml'));
    fs.copyFileSync(path.join(consumer, 'Cargo.lock'), path.join(output, 'consumer.Cargo.lock'));
    report.status = 'passed'; report.phase = 'complete'; report.exitCode = 0;
  } catch (error) { report.status = 'failed'; report.error = error.message; process.exitCode = 1; }
  report.logSha256 = hash(fs.readFileSync(log));
  report.finishedAt = new Date().toISOString(); save();
  console.log(JSON.stringify({ status: report.status, phase: report.phase, packages: report.packages.length, error: report.error }));
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
