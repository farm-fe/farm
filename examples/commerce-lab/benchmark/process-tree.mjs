import { ChildProcess, execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { performance } from 'node:perf_hooks';

const execute = promisify(execFile);
export function processTree(rows, rootPid) {
  const root = rows.find(row => row.pid === rootPid);
  if (!root) return null;
  const pending = [rootPid], visited = new Set(), processes = [];
  while (pending.length) {
    const pid = pending.shift();
    if (visited.has(pid)) continue;
    visited.add(pid);
    const row = rows.find(item => item.pid === pid);
    if (!row || !Number.isSafeInteger(row.workingSetBytes) || row.workingSetBytes < 0) throw new Error('Invalid process memory observation');
    processes.push(row);
    pending.push(...rows.filter(item => item.parentPid === pid).map(item => item.pid));
  }
  processes.sort((a, b) => a.pid - b.pid);
  return { processes, workingSetBytes: processes.reduce((sum, row) => sum + row.workingSetBytes, 0) };
}

export async function snapshotWindowsTree(rootPid) {
  if (process.platform !== 'win32') throw new Error('Windows process snapshot requested on another platform');
  if (!Number.isSafeInteger(rootPid) || rootPid <= 0) throw new Error('Invalid rootPid');
  const started = performance.now();
  const script = "Get-CimInstance Win32_Process | Select-Object @{n='pid';e={[int]$_.ProcessId}},@{n='parentPid';e={[int]$_.ParentProcessId}},@{n='workingSetBytes';e={[long]$_.WorkingSetSize}},@{n='createdAt';e={$_.CreationDate.ToUniversalTime().ToString('o')}} | ConvertTo-Json -Compress";
  const { stdout } = await execute('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command', script], { windowsHide: true, timeout: 15000, maxBuffer: 8 * 1024 * 1024 });
  const decoded = JSON.parse(stdout.trim());
  const tree = processTree(Array.isArray(decoded) ? decoded : [decoded], rootPid);
  return { timestamp: new Date().toISOString(), collectionMs: performance.now() - started, tree,
    metric: 'Sum of process WorkingSetSize; shared pages may be counted more than once.' };
}

export async function terminateOwnedWindowsTree(child) {
  if (process.platform !== 'win32') throw new Error('Windows process termination requested on another platform');
  if (!(child instanceof ChildProcess) || !Number.isSafeInteger(child.pid)) throw new Error('Expected an owned ChildProcess');
  if (child.exitCode !== null || child.signalCode !== null) return { alreadyExited: true };
  const { stdout, stderr } = await execute('taskkill.exe', ['/PID', String(child.pid), '/T', '/F'], { windowsHide: true, timeout: 15000 });
  return { stdout, stderr };
}

export async function observeOwnedWindowsTree(child, intervalMs = 200) {
  if (!(child instanceof ChildProcess) || !Number.isSafeInteger(child.pid)) throw new Error('Expected an owned ChildProcess');
  if (!Number.isSafeInteger(intervalMs) || intervalMs < 10) throw new Error('Invalid observation interval');
  const observations = [], errors = [];
  let closed = child.exitCode !== null || child.signalCode !== null;
  const stop = () => { closed = true; };
  child.once('close', stop);
  let rootCreatedAt;
  try {
    while (!closed) {
      try {
        const snapshot = await snapshotWindowsTree(child.pid);
        observations.push(snapshot);
        const root = snapshot.tree?.processes.find(row => row.pid === child.pid);
        if (root) {
          if (rootCreatedAt && root.createdAt !== rootCreatedAt) throw new Error('Root PID was reused');
          rootCreatedAt = root.createdAt;
        }
      } catch (error) { errors.push(error.message); break; }
      if (!closed) await new Promise(resolve => setTimeout(resolve, intervalMs));
    }
  } finally { child.off('close', stop); }
  const observed = observations.filter(item => item.tree);
  return { rootPid: child.pid, intervalMs, observations, errors, valid: observed.length > 0 && errors.length === 0,
    peakWorkingSetBytes: observed.length ? Math.max(...observed.map(item => item.tree.workingSetBytes)) : null,
    excludesBrowser: true, instrumented: true };
}
