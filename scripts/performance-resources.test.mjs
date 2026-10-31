// cspell:words cgroup
import test from 'node:test';
import assert from 'node:assert/strict';
import { resourceSnapshot, parseDarwinVmStat, diagnosticExecArgv, nodeRuntimeDiagnostics } from './performance-resources.mjs';
import { assertPressureObservation } from './performance-supervised.mjs';
test('Darwin free pages never become available physical memory or an identity error', () => {
  const probe = resourceSnapshot({ platform: 'darwin', freeBytes: 1024, totalBytes: 16 * 1024 ** 3, diskFreeBytes: 30 * 1024 ** 3 });
  assert.equal(probe.availableBytes, null); assert.equal(probe.observedFreeBytes, 1024);
  assert.throws(() => assertPressureObservation(probe));
  assert.equal(probe.observationOnly, true);
  assertPressureObservation({ ...probe, pressure: { state: 'normal' } });
});
test('Darwin VM diagnostics retain page size/counters without double-counting reclaimable memory', () => {
  const parsed = parseDarwinVmStat('Mach Virtual Memory Statistics: (page size of 16384 bytes)\nPages free: 100.\nPages inactive: 500.\nPages purgeable: 200.\n');
  assert.equal(parsed.pageSize, 16384); assert.equal(parsed.pages['Pages inactive'], 500);
  assert.ok(!Object.hasOwn(parsed, 'availableBytes'));
  const probe = resourceSnapshot({ platform: 'darwin', freeBytes: 100 * 16384, diskFreeBytes: 30 * 1024 ** 3, darwin: parsed });
  assert.equal(probe.availableBytes, null); assert.deepEqual(probe.diagnostics, parsed);
});
test('Linux uses MemAvailable, Windows uses available physical bytes, and insufficient resources stay explicit', () => {
  const linux = resourceSnapshot({ platform: 'linux', freeBytes: 1024, totalBytes: 8 * 1024 ** 3, diskFreeBytes: 30 * 1024 ** 3, linuxMemoryInfo: 'MemAvailable: 5242880 kB\n' });
  assert.equal(linux.availableBytes, 5 * 1024 ** 3); assertPressureObservation({ ...linux, pressure: { state: 'normal' } });
  const windows = resourceSnapshot({ platform: 'win32', freeBytes: 1024, diskFreeBytes: 30 * 1024 ** 3 });
  assert.throws(() => assertPressureObservation({ ...windows, pressure: { state: 'normal' } }, { requiredAvailableBytes: 2048 }));
});

test('Node runtime diagnostics retain actual heap/limits and omit arbitrary argv/env values', () => {
  const safe = diagnosticExecArgv(['--max-old-space-size=4096', '--import', '/private/module.mjs', '--token=secret', '-e', 'secret code']);
  assert.deepEqual(safe, ['--max-old-space-size=4096', '--import', '[argument omitted]', '[argument omitted]', '[argument omitted]', '[argument omitted]']);
  assert.doesNotMatch(JSON.stringify(safe), /secret|private/);
  const probe = nodeRuntimeDiagnostics({ platform: 'linux', execArgv: [], heap: { heap_size_limit: 123456 }, constrainedBytes: 456789,
    read: file => file.endsWith('memory.max') ? '8589934592' : file.endsWith('memory.high') ? 'max' : (() => { throw Object.assign(new Error('private error'), { code: 'ENOENT' }); })() });
  assert.equal(probe.heap_size_limit, 123456); assert.equal(probe.constrainedMemoryBytes, 456789);
  assert.deepEqual(probe.cgroup['/sys/fs/cgroup/memory.max'], { bytes: 8589934592 });
  assert.deepEqual(probe.cgroup['/sys/fs/cgroup/memory.high'], { unlimited: true });
  assert.doesNotMatch(JSON.stringify(probe), /private error/);
  assert.equal(nodeRuntimeDiagnostics({ platform: 'darwin', heap: { heap_size_limit: 1 }, constrainedBytes: 0 }).constrainedMemoryBytes, null);
});
