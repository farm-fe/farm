import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { assertPressureObservation, finishBootstrap, commandBudget, supervisedCommand, safetyProtocol } from './performance-supervised.mjs';

const observation = () => ({ availableBytes: null, diskFreeBytes: 30 * 1024 ** 3, pressure: { state: 'normal', source: 'fixture only' } });
const identity = { fixtureSha256: 'f'.repeat(64), harnessSha256: 'e'.repeat(64), revision: 'a'.repeat(40), os: 'darwin', arch: 'arm64' };
const rows = () => ['head', 'vite'].flatMap(cohort => ['cold', 'warm'].map(cacheMode => ({ cohort, cacheMode, success: true,
  supervision: { status: 'complete', cleanupConfirmed: true, samples: 2, durationMs: cacheMode === 'cold' ? 150000 : 60000,
    kernelPeakRssBytes: 1024, peakObservedGroupRssBytes: 2048, rssSource: 'fixture', before: observation(), after: observation() } })));

test('Darwin pressure admission preserves unknown available bytes; warning, unknown and disk failure stop admission', () => {
  assertPressureObservation(observation());
  for (const state of ['warning', 'critical', 'unknown']) assert.throws(() => assertPressureObservation({ ...observation(), pressure: { state } }));
  assert.throws(() => assertPressureObservation({ ...observation(), diskFreeBytes: 1 }));
  assert.throws(() => assertPressureObservation({ ...observation(), availableBytes: undefined }));
});

test('only complete successful cold and warm bootstrap pairs calibrate this exact fixture/run', () => {
  const budget = finishBootstrap(rows(), identity);
  assert.equal(budget.status, 'passed'); assert.equal(budget.businessBudgetCalibrated, true);
  assert.equal(budget.identity.arch, 'arm64'); assert.equal(budget.bootstrap.length, 4);
  assert.throws(() => finishBootstrap(rows().slice(0, 3), identity));
  for (const field of ['cleanupConfirmed', 'samples', 'kernelPeakRssBytes', 'durationMs']) {
    const records = rows(); records[0].supervision[field] = 0; assert.throws(() => finishBootstrap(records, identity), field);
  }
  const records = rows(); records[0].success = false; assert.throws(() => finishBootstrap(records, identity));
  assert.throws(() => finishBootstrap(rows(), {}));
});

test('Windows 120-second ceiling is replaced by a bounded bootstrap allocation, not unlimited retries', () => {
  const now = 1000, deadline = now + 18 * 60 * 1000;
  assert.equal(commandBudget({ phase: 'bootstrap', cacheMode: 'cold', now, deadline }).timeoutMs, 240000);
  assert.equal(commandBudget({ phase: 'bootstrap', cacheMode: 'warm', now, deadline }).timeoutMs, 180000);
  assert.throws(() => commandBudget({ phase: 'sampling', now, deadline }));
  const budget = finishBootstrap(rows(), identity);
  const allocated = commandBudget({ phase: 'graph', budget, now, deadline });
  assert.equal(allocated.timeoutMs, 235000);
  assert.equal(allocated.policy.requiredAvailableBytes, 4096);
  assert.equal(commandBudget({ phase: 'graph', budget, now, deadline: now + 20000 }).timeoutMs, 5000);
  assert.throws(() => commandBudget({ phase: 'graph', budget: { ...budget, protocol: 'old' }, now, deadline }));
  assert.throws(() => commandBudget({ phase: 'graph', budget, now, deadline: now + 10000 }));
  assert.equal(budget.protocol, safetyProtocol);
});

test('real OS supervisor reports root exit and owned descendant cleanup, including timeout', { skip: !process.env.FARM_PERFORMANCE_PYTHON, timeout: 45000 }, async () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-supervisor-contract-'));
  let cleanupConfirmed = true;
  try {
    const options = { cwd: directory, log: path.join(directory, 'child.log'), phase: 'bootstrap', deadline: Date.now() + 40000 };
    const result = await supervisedCommand(process.execPath, ['-e', 'setTimeout(()=>process.exit(0),200)'], options);
    assert.equal(result.cleanupConfirmed, true); assert.ok(result.kernelPeakRssBytes > 0 && result.durationMs > 0);
    const pidFile = path.join(directory, 'owned-child.json');
    const source = `const cp=require('node:child_process').spawn(process.execPath,['-e','setInterval(()=>{},1000)'],{stdio:'ignore'});require('node:fs').writeFileSync(process.argv[1],JSON.stringify({parent:process.pid,child:cp.pid}));setInterval(()=>{},1000);`;
    await assert.rejects(supervisedCommand(process.execPath, ['-e', source, pidFile], { ...options, commandLimitMs: 1500 }), error => {
      cleanupConfirmed = error.cleanupConfirmed === true;
      assert.equal(error.cleanupConfirmed, true); assert.equal(error.supervision.status, 'invalid'); return true;
    });
    const pids = JSON.parse(fs.readFileSync(pidFile));
    cleanupConfirmed = false;
    for (const pid of Object.values(pids)) assert.throws(() => process.kill(pid, 0), error => error.code === 'ESRCH');
    cleanupConfirmed = true;
  } catch (error) {
    if (error.cleanupConfirmed === false) cleanupConfirmed = false;
    throw error;
  } finally { if (cleanupConfirmed) fs.rmSync(directory, { recursive: true, force: true }); else console.error('Unconfirmed cleanup; fixture preserved: ' + directory); }
});
