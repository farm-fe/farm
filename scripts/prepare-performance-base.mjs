// cspell:words statfs bavail
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import { fileURLToPath } from 'node:url';
import { discoverBaseCohort, prepareBaseConsumer } from './performance-cohorts.mjs';
import { command, protocol } from './performance-report.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const baseRoot = path.join(root, '.performance-base');
const receiptFile = path.join(root, '.performance-base-receipt.json');
const log = path.join(root, '.performance-base.log');
let receipt = { mode: 'current-only', reason: 'Base source is not compatible with this protocol' };
try {
  const source = await fs.readFile(path.join(baseRoot, 'scripts/performance-report.mjs'), 'utf8');
  if (source.includes(`'${protocol}'`)) {
    const cohort = await discoverBaseCohort({ repository: process.env.GITHUB_REPOSITORY,
      revision: process.env.FARM_BASE_SHA, abi: process.env.FARM_PERFORMANCE_ABI });
    receipt = await prepareBaseConsumer({ baseRoot, revision: process.env.FARM_BASE_SHA, cohort, protocol });
    if (receipt.mode === 'paired') {
      let executable = 'pnpm';
      let prefix = [];
      if (process.platform === 'win32') {
        // Execute the installed CLI with Node; do not hand .cmd arguments to a shell.
        let cli;
        for (const directory of (process.env.PATH ?? '').split(path.delimiter)) {
          for (const relative of ['../pnpm/bin/pnpm.cjs', 'node_modules/pnpm/bin/pnpm.cjs']) {
            const candidate = path.resolve(directory, relative);
            try { await fs.access(candidate); cli = candidate; break; } catch {}
          }
          if (cli) break;
        }
        if (!cli) throw new Error('Installed Windows pnpm CLI could not be resolved');
        executable = process.execPath; prefix = [cli];
      }
      // Source and native cohorts stay separate. No Cargo or head native fallback.
      const steps = [['install', '--frozen-lockfile'],
        ['--filter', '@farmfe/cli', 'build'], ['--filter', '@farmfe/runtime', 'build'],
        ['--filter', '@farmfe/runtime-plugin-hmr', 'build'],
        ['--filter', '@farmfe/runtime-plugin-import-meta', 'build'],
        ['--filter', '@farmfe/core', 'build:cjs']];
      for (const args of steps) {
        const disk = await fs.statfs(baseRoot);
        if (os.freemem() < 3.4 * 1024 ** 3 || disk.bavail * disk.bsize < 8 * 1024 ** 3) throw new Error('Resource guard stopped base JavaScript preparation');
        const remaining = Number(process.env.FARM_PERFORMANCE_STARTED_AT ?? Date.now()) + 18 * 60 * 1000 - Date.now();
        if (remaining <= 0) throw new Error('Performance job budget exhausted during base preparation');
        await command(executable, [...prefix, ...args], { cwd: baseRoot, env: process.env, log, timeoutMs: Math.min(120000, remaining) });
      }
    }
  }
} catch (error) { receipt = { mode: 'current-only', reason: `Base preparation unavailable: ${error.message}` }; }
await fs.writeFile(receiptFile, JSON.stringify(receipt, null, 2));
console.log(JSON.stringify(receipt));
