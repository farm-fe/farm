// cspell:words cgroup jitless
import fs from 'node:fs';
import os from 'node:os';
import { getHeapStatistics } from 'node:v8';
import { execFileSync } from 'node:child_process';
export function parseDarwinVmStat(text) {
  const pageSize = Number(text.match(/page size of (\d+) bytes/)?.[1]);
  if (!Number.isSafeInteger(pageSize) || pageSize <= 0) throw new Error('Unknown Darwin VM page size');
  return { pageSize, pages: Object.fromEntries([...text.matchAll(/^([^:\n]+):\s+(\d+)\./gm)].map(match => [match[1], Number(match[2])])),
    limitation: 'Raw VM counters only; inactive/purgeable/speculative categories are not summed into invented available memory' };
}
function darwinDiagnostics() {
  const observations = {};
  for (const [name, file, args] of [['vmStat', '/usr/bin/vm_stat', []], ['pressure', '/usr/bin/memory_pressure', ['-Q']]]) {
    try { const output = execFileSync(file, args, { encoding: 'utf8', timeout: 2000, maxBuffer: 16384, stdio: ['ignore', 'pipe', 'pipe'] }); observations[name] = name === 'vmStat' ? parseDarwinVmStat(output) : { queryOutput: output.trim() }; }
    catch (error) { observations[name] = { unavailable: true, code: error.code ?? error.status }; }
  }
  return observations;
}
export function resourceSnapshot({ platform = process.platform, freeBytes = os.freemem(), totalBytes = os.totalmem(), diskFreeBytes, linuxMemoryInfo, darwin } = {}) {
  let availableBytes = null, method, reason = null;
  if (platform === 'win32') { availableBytes = freeBytes; method = 'Windows available physical memory via libuv'; }
  else if (platform === 'linux') {
    let text = linuxMemoryInfo;
    try { text ??= fs.readFileSync('/proc/meminfo', 'utf8'); } catch (error) { reason = `MemAvailable probe failed: ${error.code}`; }
    const value = text?.match(/^MemAvailable:\s+(\d+) kB$/m);
    if (value) availableBytes = Number(value[1]) * 1024;
    method = 'Linux MemAvailable';
    if (availableBytes === null) reason ??= 'MemAvailable probe is unavailable';
  } else {
    method = 'Observed free pages only; available memory is not established';
    reason = 'Raw Darwin free pages are not admission evidence. The OS supervisor must verify pressure and complete a full workload bootstrap.';
  }
  return { platform, observedFreeBytes: freeBytes, totalBytes, availableBytes, diskFreeBytes, method, reason,
    ...(platform === 'darwin' ? { diagnostics: darwin ?? (process.platform === 'darwin' ? darwinDiagnostics() : null) } : {}),
    observationOnly: true, policy: { admission: 'Verified OS pressure plus supervised cold/warm bootstrap required', diskFloorBytes: 8 * 1024 ** 3, businessBudgetCalibrated: false } };
}

// Only numeric heap flags may keep their values. Never print arbitrary runtime
// arguments, script/eval contents, environment variables or application paths.
export function diagnosticExecArgv(values) {
  const numeric = /^--(?:max[-_]old[-_]space[-_]size|max[-_]semi[-_]space[-_]size|stack[-_]size)=\d+$/;
  const names = new Set(['--import', '--require', '--jitless', '--no-lazy', '--expose-gc', '--trace-gc']);
  return values.slice(0, 40).map(value => typeof value === 'string' && value.length <= 120 && numeric.test(value) ? value : names.has(value) ? value : '[argument omitted]');
}
export function nodeRuntimeDiagnostics({ platform = process.platform, execArgv = process.execArgv,
  heap = getHeapStatistics(), constrainedBytes = process.constrainedMemory?.(), read = file => fs.readFileSync(file, 'utf8') } = {}) {
  const cgroup = {};
  if (platform === 'linux') for (const file of ['/sys/fs/cgroup/memory.max', '/sys/fs/cgroup/memory.high', '/sys/fs/cgroup/memory.current', '/sys/fs/cgroup/memory/memory.limit_in_bytes']) {
    try {
      const value = read(file).trim();
      cgroup[file] = value === 'max' ? { unlimited: true } : /^\d+$/.test(value) && Number.isSafeInteger(Number(value)) ? { bytes: Number(value) } : { unavailable: true, reason: 'Non-numeric limit' };
    } catch (error) { cgroup[file] = { unavailable: true, code: /^[A-Z_]+$/.test(error.code ?? '') ? error.code : 'UNKNOWN' }; }
  }
  return { node: process.version, pid: process.pid, platform, heap_size_limit: heap.heap_size_limit,
    execArgv: diagnosticExecArgv(execArgv), constrainedMemoryBytes: Number.isFinite(constrainedBytes) && constrainedBytes > 0 ? constrainedBytes : null,
    cgroup, scope: 'Actual Node root heap limit and OS/container observations; not total process-tree RSS or an OOM diagnosis' };
}
// Supervised Node commands preload this exact module through an explicit argv
// flag. The marker enables only the bounded whitelist record, not an env dump.
if (process.env.FARM_PERFORMANCE_RUNTIME_DIAGNOSTIC === '1') process.stderr.write('[performance node runtime] ' + JSON.stringify(nodeRuntimeDiagnostics()) + '\n');
