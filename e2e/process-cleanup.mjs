import { execFile } from 'node:child_process';
import { readlinkSync } from 'node:fs';
import { promisify } from 'node:util';
import { setTimeout as delay } from 'node:timers/promises';
import { logger } from './utils.mjs';

const execFileAsync = promisify(execFile);
const DEFAULT_STALE_PROCESS_SECONDS = 5 * 60;

const configuredStaleProcessSeconds = Number.parseInt(
  process.env.FARM_E2E_STALE_PROCESS_SECONDS ?? '',
  10
);

const STALE_E2E_PROCESS_SECONDS = Number.isFinite(configuredStaleProcessSeconds)
  && configuredStaleProcessSeconds >= 0
  ? configuredStaleProcessSeconds
  : DEFAULT_STALE_PROCESS_SECONDS;

/**
 * @typedef {{ pid: number, ppid: number, processGroupId: number, elapsedSeconds: number, command: string }} ProcessInfo
 */

/**
 * @param {string} value
 * @returns {string}
 */
function normalizePath(value) {
  return value.replace(/\\/g, '/');
}

/**
 * @param {string} value
 * @returns {number}
 */
function parseElapsedSeconds(value) {
  const [dayPart, timePart = dayPart] = value.includes('-')
    ? value.split('-', 2)
    : ['', value];
  const days = dayPart ? Number(dayPart) : 0;
  const parts = timePart.split(':').map(Number);

  if (parts.some((part) => Number.isNaN(part))) {
    return 0;
  }

  if (parts.length === 2) {
    const [minutes, seconds] = parts;
    return days * 86_400 + minutes * 60 + seconds;
  }

  if (parts.length === 3) {
    const [hours, minutes, seconds] = parts;
    return days * 86_400 + hours * 3600 + minutes * 60 + seconds;
  }

  return 0;
}

/**
 * @param {string} output
 * @returns {ProcessInfo[]}
 */
function parseUnixProcessList(output) {
  return output
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const match = line.match(/^(\d+)\s+(\d+)\s+(\d+)\s+(\S+)\s+(.+)$/);
      if (!match) return null;
      return {
        pid: Number(match[1]),
        ppid: Number(match[2]),
        processGroupId: Number(match[3]),
        elapsedSeconds: parseElapsedSeconds(match[4]),
        command: match[5]
      };
    })
    .filter(Boolean);
}

/**
 * @returns {Promise<ProcessInfo[]>}
 */
async function listUnixProcesses() {
  const { stdout } = await execFileAsync('ps', ['-axo', `pid=,ppid=,p${'gid'}=,etime=,command=`], {
    encoding: 'utf8',
    maxBuffer: 1024 * 1024 * 10
  });

  return parseUnixProcessList(stdout);
}

/**
 * @param {unknown} value
 * @returns {ProcessInfo[]}
 */
function parseWindowsProcessList(value) {
  const entries = Array.isArray(value) ? value : [value];
  return entries
    .filter(Boolean)
    .map((entry) => {
      const pid = Number(entry.ProcessId);
      const ppid = Number(entry.ParentProcessId);
      const elapsedSeconds = Number(entry.ElapsedSeconds);
      if (!Number.isFinite(pid) || !Number.isFinite(ppid)) {
        return null;
      }

      return {
        pid,
        ppid,
        processGroupId: pid,
        elapsedSeconds: Number.isFinite(elapsedSeconds) ? elapsedSeconds : 0,
        command: String(entry.CommandLine || entry.ExecutablePath || '')
      };
    })
    .filter(Boolean);
}

/**
 * @returns {Promise<ProcessInfo[]>}
 */
async function listWindowsProcesses() {
  const command = `
$ErrorActionPreference = 'Stop'
Get-CimInstance Win32_Process |
  Select-Object ProcessId, ParentProcessId,
    @{Name='ElapsedSeconds';Expression={[int]((Get-Date) - $_.CreationDate).TotalSeconds}},
    CommandLine, ExecutablePath |
  ConvertTo-Json -Compress
`;
  let stdout;

  try {
    ({ stdout } = await execFileAsync('powershell.exe', [
      '-NoProfile',
      '-NonInteractive',
      '-Command',
      command
    ], {
      encoding: 'utf8',
      maxBuffer: 1024 * 1024 * 10
    }));
  } catch {
    ({ stdout } = await execFileAsync('powershell', [
      '-NoProfile',
      '-NonInteractive',
      '-Command',
      command
    ], {
      encoding: 'utf8',
      maxBuffer: 1024 * 1024 * 10
    }));
  }

  return parseWindowsProcessList(JSON.parse(stdout || '[]'));
}

/**
 * @returns {Promise<ProcessInfo[]>}
 */
async function listProcesses() {
  return process.platform === 'win32'
    ? listWindowsProcesses()
    : listUnixProcesses();
}

/**
 * @param {ProcessInfo} proc
 * @param {string} root
 * @returns {boolean}
 */
function isProcessInRoot(proc, root) {
  const normalizedCommand = normalizePath(proc.command);
  const normalizedRoot = normalizePath(root);

  if (normalizedCommand.includes(`${normalizedRoot}/`)) {
    return true;
  }

  if (process.platform === 'linux') {
    try {
      const cwd = normalizePath(readlinkSync(`/proc/${proc.pid}/cwd`));
      return cwd === normalizedRoot || cwd.startsWith(`${normalizedRoot}/`);
    } catch {}
  }

  return false;
}

/**
 * @param {ProcessInfo} proc
 * @param {string} root
 * @returns {boolean}
 */
function isFarmE2EProcess(proc, root) {
  const command = normalizePath(proc.command);
  if (!isProcessInRoot(proc, root)) {
    return false;
  }

  return (
    /\bscripts\/test-e2e\.mjs\b/.test(command) ||
    /\bscripts\/test-e2e-worker\.mjs\b/.test(command) ||
    /\bfarm\.mjs\s+(start|preview|build)\b/.test(command) ||
    /\bpnpm(\.cmd|\.ps1)?\s+test-e2e\b/.test(command)
  );
}

/**
 * @param {Map<number, ProcessInfo>} processByPid
 * @param {ProcessInfo} proc
 * @returns {number[]}
 */
function ancestorProcessIds(processByPid, proc) {
  const ancestors = [];
  const seen = new Set([proc.pid]);
  let current = proc;

  while (current.ppid > 1 && !seen.has(current.ppid)) {
    const parent = processByPid.get(current.ppid);
    if (!parent) break;
    ancestors.push(parent.pid);
    seen.add(parent.pid);
    current = parent;
  }

  return ancestors;
}

/**
 * @param {Map<number, ProcessInfo>} processByPid
 * @param {ProcessInfo} proc
 * @returns {number}
 */
function processDepth(processByPid, proc) {
  return ancestorProcessIds(processByPid, proc).length;
}

/**
 * @param {number} processGroupId
 * @returns {void}
 */
function killUnixProcessGroup(processGroupId) {
  try {
    process.kill(-processGroupId, 'SIGTERM');
  } catch {}
}

/**
 * @param {number} processGroupId
 * @returns {void}
 */
function forceKillUnixProcessGroup(processGroupId) {
  try {
    process.kill(-processGroupId, 'SIGKILL');
  } catch {}
}

/**
 * @param {number} pid
 * @returns {Promise<void>}
 */
async function killWindowsProcessTree(pid) {
  try {
    await execFileAsync(`task${'kill'}.exe`, ['/PID', String(pid), '/T', '/F'], {
      encoding: 'utf8'
    });
  } catch {}
}

/**
 * @param {{ stage: 'before' | 'after', includeCurrentRun?: boolean }} options
 * @returns {Promise<void>}
 */
export async function cleanupStaleE2EProcesses({ stage, includeCurrentRun = false }) {
  let processes;
  try {
    processes = await listProcesses();
  } catch (error) {
    logger(`Unable to inspect stale E2E processes: ${error}`, { color: 'yellow' });
    return;
  }

  const root = process.cwd();
  const processByPid = new Map(processes.map((proc) => [proc.pid, proc]));
  const currentAncestors = new Set(ancestorProcessIds(processByPid, {
    pid: process.pid,
    ppid: process.ppid,
    processGroupId: process.pid,
    elapsedSeconds: 0,
    command: process.argv.join(' ')
  }));
  const currentProcess = processByPid.get(process.pid);
  const currentProcessGroupId = currentProcess?.processGroupId;
  const currentRunProcessIds = new Set([process.pid, process.ppid, ...currentAncestors]);
  const farmProcessIds = new Set(
    processes
      .filter((proc) => isFarmE2EProcess(proc, root))
      .map((proc) => proc.pid)
  );
  const staleProcesses = [];

  for (const proc of processes) {
    const isCurrentProcessGroup = process.platform !== 'win32'
      && currentProcessGroupId != null
      && proc.processGroupId === currentProcessGroupId;
    const isCurrentProcess = currentRunProcessIds.has(proc.pid);
    if (!includeCurrentRun && (isCurrentProcessGroup || isCurrentProcess)) {
      continue;
    }

    const command = normalizePath(proc.command);
    const ancestors = ancestorProcessIds(processByPid, proc);
    const belongsToFarmE2E = isFarmE2EProcess(proc, root)
      || ancestors.some((pid) => farmProcessIds.has(pid))
      || (command.includes('/ms-playwright/') && (
        ancestors.some((pid) => farmProcessIds.has(pid)) || proc.ppid === 1
      ));

    if (!belongsToFarmE2E || proc.elapsedSeconds < STALE_E2E_PROCESS_SECONDS) {
      continue;
    }

    staleProcesses.push(proc);
  }

  if (staleProcesses.length === 0) {
    return;
  }

  if (process.platform === 'win32') {
    const staleRoots = staleProcesses
      .filter((proc) => !staleProcesses.some((candidate) => proc.ppid === candidate.pid))
      .sort((a, b) => processDepth(processByPid, a) - processDepth(processByPid, b));
    const summary = staleRoots.map((proc) => String(proc.pid)).join(', ');
    logger(`Cleaning stale E2E process trees ${stage} run: ${summary}`, { color: 'yellow' });

    for (const proc of staleRoots) {
      await killWindowsProcessTree(proc.pid);
    }
    return;
  }

  /** @type {Map<number, ProcessInfo[]>} */
  const staleGroups = new Map();
  for (const proc of staleProcesses) {
    const group = staleGroups.get(proc.processGroupId) ?? [];
    group.push(proc);
    staleGroups.set(proc.processGroupId, group);
  }

  const summary = [...staleGroups]
    .map(([processGroupId, group]) => `${processGroupId} (${group.length} process${group.length === 1 ? '' : 'es'})`)
    .join(', ');
  logger(`Cleaning stale E2E process groups ${stage} run: ${summary}`, { color: 'yellow' });

  for (const processGroupId of staleGroups.keys()) {
    killUnixProcessGroup(processGroupId);
  }

  await delay(1000);

  for (const processGroupId of staleGroups.keys()) {
    forceKillUnixProcessGroup(processGroupId);
  }
}