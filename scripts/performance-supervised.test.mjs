// cspell:words pids importlib dont
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { assertPressureObservation, finishBootstrap, commandBudget, supervisedCommand, safetyProtocol, performanceDeadline, performanceTotalBudgetMs, performanceJobTimeoutMinutes } from './performance-supervised.mjs';

const observation = () => ({ availableBytes: null, diskFreeBytes: 30 * 1024 ** 3, pressure: { state: 'normal', source: 'fixture only' } });
const identity = { fixtureSha256: 'f'.repeat(64), harnessSha256: 'e'.repeat(64), revision: 'a'.repeat(40), os: 'darwin', arch: 'arm64' };
const rows = () => ['head', 'base'].flatMap(cohort => ['cold', 'warm'].map(cacheMode => ({ cohort, cacheMode, success: true,
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
    const runtimeLine = fs.readFileSync(options.log, 'utf8').split('\n').find(line => line.startsWith('[performance node runtime] '));
    assert.ok(runtimeLine, 'Actual supervised Node preload emits bounded runtime evidence');
    const runtime = JSON.parse(runtimeLine.slice('[performance node runtime] '.length));
    assert.ok(Number.isFinite(runtime.heap_size_limit) && runtime.heap_size_limit > 0);
    assert.doesNotMatch(JSON.stringify(runtime.execArgv), /setTimeout|process.exit/);
    const pidFile = path.join(directory, 'owned-child.json');
    const source = `const cp=require('node:child_process').spawn(process.execPath,['-e','setInterval(()=>{},1000)'],{stdio:'ignore'});require('node:fs').writeFileSync(process.argv[1],JSON.stringify({parent:process.pid,child:cp.pid}));setInterval(()=>{},1000);`;
    await assert.rejects(supervisedCommand(process.execPath, ['-e', source, pidFile], { ...options, commandLimitMs: 1500 }), error => {
      cleanupConfirmed = error.cleanupConfirmed === true;
      assert.equal(error.cleanupConfirmed, true); assert.equal(error.supervision.status, 'invalid');
      if (process.platform !== 'win32') {
        const proof = error.supervision.cleanupScopes.find(row => row.scope === 'root');
        assert.equal(proof.ownershipVerified, true); assert.equal(proof.rootReaped, true);
        assert.deepEqual(proof.remaining, []); assert.deepEqual(proof.groupMembers, []);
      }
      return true;
    });
    const pids = JSON.parse(fs.readFileSync(pidFile));
    cleanupConfirmed = false;
    for (const pid of Object.values(pids)) assert.throws(() => process.kill(pid, 0), error => error.code === 'ESRCH');
    cleanupConfirmed = true;
    // Playwright creates a distinct Unix browser group. A plain root-group
    // cleanup would leave this actual child alive and must not pass this test.
    const browserPidFile = path.join(directory, 'detached-child.json');
    const detached = `const fs=require('node:fs');const cp=require('node:child_process').spawn(process.execPath,['-e','setTimeout(()=>process.exit(0),30000)'],{stdio:'ignore',detached:process.platform!=='win32'});const data={rootPid:process.pid,browserPid:cp.pid,state:'open'};const receipt=process.env.FARM_PERFORMANCE_BROWSER_OWNER;fs.writeFileSync(receipt+'.tmp',JSON.stringify(data),{flag:'wx'});fs.renameSync(receipt+'.tmp',receipt);fs.writeFileSync(process.argv[1],JSON.stringify(data));setTimeout(()=>process.exit(0),31000);`;
    await assert.rejects(supervisedCommand(process.execPath, ['-e', detached, browserPidFile], { ...options, phase: 'correctness', budget: finishBootstrap(rows(), identity), commandLimitMs: 1500 }), error => {
      cleanupConfirmed = error.cleanupConfirmed === true;
      assert.equal(error.cleanupConfirmed, true); return true;
    });
    const ownedBrowser = JSON.parse(fs.readFileSync(browserPidFile));
    cleanupConfirmed = false;
    for (const pid of [ownedBrowser.rootPid, ownedBrowser.browserPid]) assert.throws(() => process.kill(pid, 0), error => error.code === 'ESRCH');
    cleanupConfirmed = true;
  } catch (error) {
    if (error.cleanupConfirmed === false) cleanupConfirmed = false;
    throw error;
  } finally { if (cleanupConfirmed) fs.rmSync(directory, { recursive: true, force: true }); else console.error('Unconfirmed cleanup; fixture preserved: ' + directory); }
});

test('Farm-only base preparation and sampling share a finite 75-minute deadline inside the 90-minute job', () => {
  assert.equal(performanceTotalBudgetMs, 75 * 60 * 1000);
  assert.equal(performanceJobTimeoutMinutes, 90);
  assert.equal(performanceDeadline(1000), 1000 + performanceTotalBudgetMs);
  assert.ok(performanceJobTimeoutMinutes * 60000 - performanceTotalBudgetMs >= 15 * 60000);
  assert.throws(() => performanceDeadline(NaN));
  const budget = finishBootstrap(rows(), identity), now = 1000;
  assert.equal(commandBudget({ phase: 'correctness', budget, now, deadline: performanceDeadline(now) }).timeoutMs, 180000);
  const base = fs.readFileSync(new URL('./prepare-performance-base.mjs', import.meta.url), 'utf8');
  const sampler = fs.readFileSync(new URL('./performance-report.mjs', import.meta.url), 'utf8');
  assert.match(base, /performanceDeadline\(Number/); assert.match(sampler, /performanceDeadline\(startedAt\)/);
  assert.doesNotMatch(base + sampler, /18 \* 60 \* 1000/);
});

test('Linux PSI retains transient stalls but stops sustained pressure and rejects malformed counters', { skip: !process.env.FARM_PERFORMANCE_PYTHON }, () => {
  const source = String.raw`
import importlib.util, json, sys
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("supervisor_contract", sys.argv[1])
module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
def pressure(some=0, full=0, average=0):
    return module.linux_pressure({"some": {"avg10": average, "total": some}, "full": {"avg10": 0, "total": full}})
transient = module.LinuxPressureWindow(); transient.observe(pressure(), 0)
values = [transient.observe(pressure(1000, 10), i)["state"] for i in range(1, 8)]
assert values == ["normal"] * 7
reset = module.LinuxPressureWindow(); reset.observe(pressure(), 0)
for moment, total in [(1, 200000), (2, 400000), (3, 400000), (4, 400000), (5, 600000)]:
    last = reset.observe(pressure(total), moment)
assert last["state"] == "normal" and last["window"]["elevatedSamples"] == 1
sustained = module.LinuxPressureWindow(); sustained.observe(pressure(), 0)
values = [sustained.observe(pressure(i * 200000), i) for i in range(1, 7)]
assert values[0]["state"] == "normal" and values[4]["state"] == "warning"
assert values[4]["window"]["elapsedMs"] == 5000
assert values[4]["window"]["stallDeltaMicroseconds"]["some"] == 1000000
critical = module.LinuxPressureWindow(); critical.observe(pressure(), 0)
assert critical.observe(pressure(600000, 600000), 1)["state"] == "critical"
assert pressure(average=25)["state"] == "warning"
for action in [lambda: module.linux_pressure({}), lambda: module.linux_pressure({"some": {"avg10": float("nan"), "total": 0}, "full": {"avg10": 0, "total": 0}}), lambda: sustained.observe(pressure(0), 7), lambda: sustained.observe(pressure(1400000), float("nan"))]:
    try: action()
    except RuntimeError: pass
    else: raise AssertionError("Invalid pressure must fail closed")
print(json.dumps({"transient": "normal", "sustained": "warning", "critical": "critical"}))
`;
  const result = execFileSync(process.env.FARM_PERFORMANCE_PYTHON, ['-c', source, fileURLToPath(new URL('./performance-supervisor.py', import.meta.url))], { encoding: 'utf8', timeout: 10000 });
  assert.deepEqual(JSON.parse(result), { transient: 'normal', sustained: 'warning', critical: 'critical' });
});
