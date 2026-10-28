import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { spawn, execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { randomUUID } from 'node:crypto';
import { diagnosticTail, redactDiagnostic } from './v2-release-diagnostics.mjs';

export const safetyProtocol = 'business-2048-supervised-bootstrap-v1';
export const diskEmergencyReserveBytes = 8 * 1024 ** 3; // Existing workspace safety reserve, not a calibrated workload budget.
const supervisor = fileURLToPath(new URL('./performance-supervisor.py', import.meta.url));
function python(env) {
  assert.ok(env.FARM_PERFORMANCE_PYTHON && path.isAbsolute(env.FARM_PERFORMANCE_PYTHON), 'Explicit installed Python interpreter required for OS supervision');
  return env.FARM_PERFORMANCE_PYTHON;
}
export function assertPressureObservation(value, policy = {}) {
  assert.equal(value.pressure?.state, 'normal', 'Pressure warning/critical or unknown observation');
  assert.ok(Number.isFinite(value.diskFreeBytes) && value.diskFreeBytes >= (policy.diskFloorBytes ?? diskEmergencyReserveBytes), 'Disk safety reserve unavailable');
  if (value.availableBytes !== null) assert.ok(Number.isFinite(value.availableBytes), 'Available memory probe malformed');
  if (policy.requiredAvailableBytes && value.availableBytes !== null) assert.ok(value.availableBytes >= policy.requiredAvailableBytes, 'Measured memory budget unavailable');
}
export function probeSupervisedResources(cwd, env = process.env) {
  const result = JSON.parse(execFileSync(python(env), [supervisor, '--probe', cwd], { env, encoding: 'utf8', windowsHide: true, timeout: 10000, maxBuffer: 65536 }));
  assertPressureObservation(result); return result;
}
export function finishBootstrap(records, identity) {
  assert.ok(identity?.fixtureSha256 && identity?.harnessSha256 && identity?.revision && identity?.os && identity?.arch, 'Bootstrap must bind the actual fixture, harness, revision and runner');
  assert.ok(records.length >= 2 && records.length % 2 === 0, 'Complete cold/warm bootstrap pairs required');
  assert.ok(records.every(row => ['head', 'base', 'vite'].includes(row.cohort) && ['cold', 'warm'].includes(row.cacheMode)), 'Known bootstrap cohort and cache mode required');
  for (const cohort of new Set(records.map(row => row.cohort))) for (const mode of ['cold', 'warm']) {
    const values = records.filter(row => row.cohort === cohort && row.cacheMode === mode);
    assert.equal(values.length, 1); assert.equal(values[0].success, true);
  }
  for (const row of records) {
    const value = row.supervision;
    assert.equal(value?.status, 'complete'); assert.equal(value.cleanupConfirmed, true);
    for (const key of ['samples', 'kernelPeakRssBytes', 'peakObservedGroupRssBytes', 'durationMs']) assert.ok(Number.isFinite(value[key]) && value[key] > 0, `Actual ${key} evidence required`);
    assertPressureObservation(value.before); assertPressureObservation(value.after);
  }
  return { protocol: safetyProtocol, status: 'passed', businessBudgetCalibrated: true, identity,
    scope: 'Current runner/run/fixture only; not a reusable cross-run capacity certification',
    kernelPeakRssBytes: Math.max(...records.map(row => row.supervision.kernelPeakRssBytes)),
    sampledGroupPeakRssBytes: Math.max(...records.map(row => row.supervision.peakObservedGroupRssBytes)),
    observedDiskGrowthBytes: records.reduce((sum, row) => sum + Math.max(0, row.supervision.before.diskFreeBytes - row.supervision.after.diskFreeBytes), 0),
    bootstrap: records.map(row => ({ cohort: row.cohort, cacheMode: row.cacheMode, durationMs: row.supervision.durationMs, rssSource: row.supervision.rssSource, before: row.supervision.before, after: row.supervision.after })),
    limitation: 'Group RSS is sampled at 1 Hz and sums shared pages; kernel max RSS is separately reported, never footprint or commit bytes' };
}
export function commandBudget({ phase, cacheMode, budget, deadline, now = Date.now() }) {
  assert.ok(['bootstrap', 'base-preparation', 'sampling', 'graph', 'comparison', 'correctness'].includes(phase), 'Explicit supervised phase required');
  if (['sampling', 'graph', 'comparison', 'correctness'].includes(phase)) {
    assert.equal(budget?.status, 'passed', 'Successful full cold/warm bootstrap required before measured work');
    assert.equal(budget.protocol, safetyProtocol, 'Bootstrap safety protocol mismatch');
  }
  const remaining = deadline - now - 15000;
  assert.ok(remaining > 0, 'Total performance deadline exhausted; reserve cleanup time');
  const initialCap = cacheMode === 'warm' ? 180000 : 240000;
  const measured = budget?.bootstrap?.filter(row => row.cacheMode === (cacheMode ?? 'cold')).map(row => row.durationMs) ?? [];
  const empiricalCap = measured.length ? Math.ceil(Math.max(...measured) * 1.5 + 10000) : initialCap;
  return { timeoutMs: Math.min(remaining, phase === 'comparison' ? 120000 : Math.min(initialCap, empiricalCap)),
    policy: { diskFloorBytes: diskEmergencyReserveBytes + (budget ? budget.observedDiskGrowthBytes : 0),
      requiredAvailableBytes: budget ? Math.ceil(Math.max(budget.kernelPeakRssBytes, budget.sampledGroupPeakRssBytes) * 2) : null,
      admission: budget ? 'same-run measured RSS x2 and observed disk growth; pressure watchdog remains authoritative' : 'supervised bootstrap, no invented available-memory floor' } };
}
export async function supervisedCommand(executable, args, { cwd, env = process.env, log, phase, cacheMode, budget, deadline, trace, commandLimitMs } = {}) {
  const allocation = commandBudget({ phase, cacheMode, budget, deadline });
  if (commandLimitMs !== undefined) {
    assert.ok(Number.isFinite(commandLimitMs) && commandLimitMs >= 500, 'Optional command limit may only narrow the supervised budget');
    allocation.timeoutMs = Math.min(allocation.timeoutMs, commandLimitMs);
  }
  const metadata = { phase, cacheMode, cwd, timeoutMs: allocation.timeoutMs, safetyProtocol };
  console.log('[performance command start] ' + JSON.stringify(metadata));
  const started = Date.now(); let lastProgress = 0, buffer = '', result;
  const interpreter = python(env);
  const browserOwnerFile = phase === 'correctness' ? path.join(path.dirname(log), `browser-owner-${randomUUID()}.json`) : null;
  if (browserOwnerFile) fs.writeFileSync(browserOwnerFile, JSON.stringify({ state: 'not-started', rootPid: null, browserPid: null }), { flag: 'wx' });
  const fd = fs.openSync(log, 'a');
  const child = spawn(interpreter, [supervisor], { env: browserOwnerFile ? { ...env, FARM_PERFORMANCE_BROWSER_OWNER: browserOwnerFile } : env, windowsHide: true, stdio: ['pipe', 'pipe', fd] });
  child.stdin.end(JSON.stringify({ argv: [executable, ...args], cwd, log, phase, browserOwnerFile, timeoutMs: allocation.timeoutMs, policy: allocation.policy }) + '\n');
  let timer, stopTimer, failure;
  try {
    await new Promise((resolve, reject) => {
      child.on('error', reject);
      child.stdin.on('error', error => { failure ??= error; });
      const stop = error => {
        failure ??= error;
        if (stopTimer) return;
        child.kill('SIGTERM');
        stopTimer = setTimeout(() => reject(new Error('Supervisor cleanup result unavailable after stop request')), 5000);
      };
      child.stdout.on('data', bytes => {
        buffer += bytes.toString();
        if (buffer.length > 65536) { stop(new Error('Supervisor message exceeds bound')); return; }
        let index;
        while ((index = buffer.indexOf('\n')) >= 0) {
          const line = buffer.slice(0, index); buffer = buffer.slice(index + 1);
          try {
            const message = JSON.parse(line);
            if (message.event === 'result') result = message;
            else if (message.event === 'sample') {
              // The measured capacity margin is checked before allocation; do
              // not demand the same unused bytes while the child consumes them.
              assertPressureObservation(message, { diskFloorBytes: allocation.policy.diskFloorBytes });
              if (trace) fs.appendFileSync(trace, JSON.stringify({ ...metadata, ...message }) + '\n');
              if (Date.now() - lastProgress >= 10000) {
                lastProgress = Date.now(); console.log('[performance command progress] ' + JSON.stringify({ ...metadata, elapsedMs: message.elapsedMs, rssBytes: message.rssBytes, liveProcessCpuSeconds: message.liveProcessCpuSeconds, pressure: message.pressure, diskFreeBytes: message.diskFreeBytes }));
              }
            } else throw new Error('Unknown supervisor message');
          } catch (error) { stop(error); }
        }
      });
      child.once('close', code => failure ? reject(failure) : code === 0 ? resolve() : reject(new Error(result?.error ?? `Supervisor exited ${code}`)));
      timer = setTimeout(() => stop(new Error('Supervisor exceeded bounded cleanup deadline')), allocation.timeoutMs + 10000);
    });
    assert.equal(result?.status, 'complete'); assert.equal(result.cleanupConfirmed, true);
    assert.equal(result.phase, phase); assert.equal(result.cwd, cwd); assert.equal(result.timeoutMs, allocation.timeoutMs); assert.equal(result.exitCode, 0);
    assert.ok(result.durationMs > 0 && result.kernelPeakRssBytes > 0, 'Actual time/RSS evidence unavailable');
    return { ...result, allocation, phase, cwd };
  } catch (cause) {
    const error = Object.assign(new Error(cause.message), { failureKind: result?.failureKind ?? 'supervised-command-failure', cleanupConfirmed: result?.cleanupConfirmed === true, supervision: result ?? { status: 'invalid', cleanupConfirmed: false } });
    try { console.error('[performance command failed] ' + JSON.stringify({ ...metadata, elapsedMs: Date.now() - started, error: redactDiagnostic(error.message, env), supervision: result, tail: diagnosticTail(log, env) })); }
    catch (diagnosticError) { console.error('[performance diagnostics unavailable] ' + redactDiagnostic(diagnosticError.message, env)); }
    throw error;
  } finally { clearTimeout(timer); clearTimeout(stopTimer); fs.closeSync(fd); }
}
