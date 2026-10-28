// cspell:words statfs bavail
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { discoverBaseCohort, prepareBaseConsumer } from './performance-cohorts.mjs';
import { protocol } from './performance-report.mjs';
import { supervisedCommand } from './performance-supervised.mjs';
import { resolvePnpmCli } from './v2-pnpm-cli.mjs';

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
      const pnpm = resolvePnpmCli();
      // Source and native cohorts stay separate. No Cargo or head native fallback.
      const steps = [['install', '--frozen-lockfile'],
        ['--filter', '@farmfe/cli', 'build'], ['--filter', '@farmfe/runtime', 'build'],
        ['--filter', '@farmfe/runtime-plugin-hmr', 'build'],
        ['--filter', '@farmfe/runtime-plugin-import-meta', 'build'],
        ['--filter', '@farmfe/core', 'build:cjs']];
      for (const args of steps) {
        const deadline = Number(process.env.FARM_PERFORMANCE_STARTED_AT ?? Date.now()) + 18 * 60 * 1000;
        await supervisedCommand(process.execPath, [pnpm, ...args], { phase: 'base-preparation', deadline, cwd: baseRoot, env: process.env, log });
      }
    }
  }
} catch (error) {
  receipt = { mode: 'current-only', reason: `Base preparation unavailable: ${error.message}`, cleanupConfirmed: error.cleanupConfirmed };
  if (error.cleanupConfirmed === false) process.exitCode = 1; // Never start head sampling with an unconfirmed base process tree.
}
await fs.writeFile(receiptFile, JSON.stringify(receipt, null, 2));
console.log(JSON.stringify(receipt));
