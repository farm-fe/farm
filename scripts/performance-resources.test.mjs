import test from 'node:test';
import assert from 'node:assert/strict';
import { resourceSnapshot, assertPerformanceResources, parseDarwinVmStat } from './performance-resources.mjs';
test('Darwin free pages never become available physical memory or an identity error', () => {
  const probe = resourceSnapshot({ platform: 'darwin', freeBytes: 1024, totalBytes: 16 * 1024 ** 3, diskFreeBytes: 30 * 1024 ** 3 });
  assert.equal(probe.availableBytes, null); assert.equal(probe.observedFreeBytes, 1024);
  assert.throws(() => assertPerformanceResources(probe), error => error.failureKind === 'resource-unavailable');
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
  assert.equal(linux.availableBytes, 5 * 1024 ** 3); assertPerformanceResources(linux);
  const windows = resourceSnapshot({ platform: 'win32', freeBytes: 1024, diskFreeBytes: 30 * 1024 ** 3 });
  assert.throws(() => assertPerformanceResources(windows), error => error.failureKind === 'resource-insufficient');
});
