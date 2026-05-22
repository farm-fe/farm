/**
 * Farm E2E test orchestrator.
 *
 * Distributes examples across N worker processes for parallel execution.
 * Each worker runs a subset of examples in its own Node.js process with
 * an isolated browser instance.
 *
 * CLI:
 *   node scripts/test-e2e.mjs
 *   node scripts/test-e2e.mjs --example react
 *   node scripts/test-e2e.mjs --from arco-pro
 *   node scripts/test-e2e.mjs --concurrency 4
 *   node scripts/test-e2e.mjs -j 2
 *   FARM_E2E_CONCURRENCY=6 node scripts/test-e2e.mjs
 */
import { existsSync, readdirSync, statSync } from 'node:fs';
import { fork } from 'node:child_process';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { cpus } from 'node:os';
import { printSummary } from '../e2e/runner.mjs';
import { cleanupStaleE2EProcesses } from '../e2e/process-cleanup.mjs';
import { logger, setLogFile, closeLogFiles } from '../e2e/utils.mjs';

// ---------------------------------------------------------------------------
// CLI argument parsing
// ---------------------------------------------------------------------------

const DEFAULT_CONCURRENCY = Math.min(4, cpus().length);

/**
 * @param {string[]} argv
 * @returns {{ example?: string, startFrom?: string, concurrency: number }}
 */
function parseArgs(argv) {
  let example;
  let startFrom;
  let concurrency = parseInt(process.env.FARM_E2E_CONCURRENCY, 10) || DEFAULT_CONCURRENCY;

  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];

    if (arg === '--example' || arg === '--project') {
      example = argv[++i];
      continue;
    }
    if (arg.startsWith('--example=')) {
      example = arg.slice('--example='.length);
      continue;
    }
    if (arg.startsWith('--project=')) {
      example = arg.slice('--project='.length);
      continue;
    }
    if (arg === '--from' || arg === '--start-from') {
      startFrom = argv[++i];
      continue;
    }
    if (arg.startsWith('--from=')) {
      startFrom = arg.slice('--from='.length);
      continue;
    }
    if (arg.startsWith('--start-from=')) {
      startFrom = arg.slice('--start-from='.length);
      continue;
    }
    if (arg === '--concurrency' || arg === '-j') {
      concurrency = parseInt(argv[++i], 10);
      continue;
    }
    if (arg.startsWith('--concurrency=')) {
      concurrency = parseInt(arg.slice('--concurrency='.length), 10);
      continue;
    }
    if (arg.startsWith('-j')) {
      concurrency = parseInt(arg.slice(2), 10);
      continue;
    }
  }

  return { example, startFrom, concurrency };
}

// ---------------------------------------------------------------------------
// Example discovery
// ---------------------------------------------------------------------------

/** @type {ReadonlySet<string>} */
const EXCLUDE_FROM_DEFAULT = new Set(['issues1433', 'nestjs']);

const EXAMPLES_DIR = resolve(process.cwd(), 'examples');

/**
 * @param {string} name
 * @returns {boolean}
 */
function isRunnableExample(name) {
  const hasSpecFile = existsSync(join(EXAMPLES_DIR, name, 'e2e.spec.mjs'));
  const hasIndexHtml = existsSync(join(EXAMPLES_DIR, name, 'index.html'));
  return hasSpecFile || (hasIndexHtml && !EXCLUDE_FROM_DEFAULT.has(name));
}

/**
 * @param {{ example?: string, startFrom?: string }} args
 * @returns {string[]}
 */
function discoverExamples(args) {
  const allNames = readdirSync(EXAMPLES_DIR)
    .filter((name) => statSync(join(EXAMPLES_DIR, name)).isDirectory())
    .sort((a, b) => a.localeCompare(b));

  if (args.example) {
    if (!allNames.includes(args.example)) {
      throw new Error(
        `--example '${args.example}' was not found under ./examples`
      );
    }
    return [args.example];
  }

  let names = allNames;

  if (args.startFrom) {
    const idx = names.indexOf(args.startFrom);
    if (idx === -1) {
      throw new Error(
        `--from '${args.startFrom}' was not found under ./examples`
      );
    }
    names = names.slice(idx);
  }

  return names;
}

// ---------------------------------------------------------------------------
// Worker pool
// ---------------------------------------------------------------------------

/**
 * Spawn a worker and run a group of examples. Returns results via IPC.
 * @param {string[]} examples
 * @param {number} index
 * @param {string} workerPath
 * @param {string} logFile
 * @param {Set<import('child_process').ChildProcess>} activeWorkers
 * @returns {Promise<Map<string, import('../e2e/runner.mjs').TestResult[]>>}
 */
function runWorker(examples, index, workerPath, logFile, activeWorkers) {
  return new Promise((resolve, reject) => {
    const worker = fork(workerPath, [], {
      stdio: 'inherit',
      env: {
        ...process.env,
        FARM_E2E_WORKER_INDEX: String(index)
      }
    });
    activeWorkers.add(worker);

    /** @type {Map<string, import('../e2e/runner.mjs').TestResult[]>} */
    const workerResults = new Map();
    let settled = false;
    let shutdownTimer;

    const settle = (fn) => {
      if (!settled) {
        settled = true;
        fn();
      }
    };

    worker.on('message', (msg) => {
      if (msg?.type === 'results') {
        // Reconstruct Error instances from the wire format produced by
        // the worker (see test-e2e-worker.mjs). IPC structured cloning
        // does not preserve Error subtypes / messages reliably.
        const results = Array.isArray(msg.results)
          ? msg.results.map((r) => {
              const error = r.errorMessage
                ? Object.assign(new Error(r.errorMessage), r.errorStack ? { stack: r.errorStack } : {})
                : undefined;
              return {
                fullName: r.fullName,
                passed: r.passed,
                skipped: r.skipped,
                duration: r.duration,
                ...(error ? { error } : {})
              };
            })
          : msg.results;
        workerResults.set(msg.example, results);
      } else if (msg?.type === 'skip') {
        // no-op — example was intentionally skipped
      } else if (msg?.type === 'error') {
        workerResults.set(msg.example, [{
          fullName: msg.example,
          passed: false,
          skipped: false,
          duration: 0,
          error: new Error(msg.error)
        }]);
      } else if (msg?.type === 'done') {
        if (worker.connected) worker.disconnect();
        shutdownTimer = setTimeout(() => {
          if (!worker.killed) worker.kill('SIGTERM');
        }, 5_000);
      } else if (msg?.type === 'fatal') {
        if (!worker.killed) worker.kill('SIGTERM');
        settle(() => reject(new Error(msg.error)));
      }
    });

    worker.on('error', (err) => {
      // Worker spawn errors are fatal for this worker, but we still
      // resolve with whatever results we already have.
      settle(() => resolve(workerResults));
    });

    worker.on('exit', (code) => {
      if (shutdownTimer) clearTimeout(shutdownTimer);
      activeWorkers.delete(worker);
      if (!settled) {
        settle(() => resolve(workerResults));
      }
    });

    // Send examples and log file path to worker
    worker.send({ type: 'run', examples, logFile });
  });
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main() {
  const args = parseArgs(process.argv.slice(2));

  // Set up log directory first so all output is captured
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const logDir = resolve(process.cwd(), 'logs', `e2e-${timestamp}`);
  const orchLogFile = join(logDir, 'orchestrator.log');
  setLogFile(orchLogFile);

  await cleanupStaleE2EProcesses({ stage: 'before' });

  if (args.example) {
    args.concurrency = 1; // single example => no parallelism
    logger(`Running E2E for single example: ${args.example}`, { color: 'cyan' });
  } else if (args.startFrom) {
    logger(`Running E2E from example: ${args.startFrom}`, { color: 'cyan' });
  }

  const exampleNames = discoverExamples(args);
  const runnableCount = exampleNames.filter(isRunnableExample).length;
  const concurrency = Math.min(args.concurrency, exampleNames.length);

  logger(`Discovered ${exampleNames.length} example(s)`, { color: 'cyan' });
  logger(`Runnable: ${runnableCount}  |  skipped: ${exampleNames.length - runnableCount}`, { color: 'cyan' });
  logger(`Running with ${concurrency} worker(s)`, { color: 'cyan' });

  // Distribute examples round-robin across workers
  /** @type {string[][]} */
  const groups = Array.from({ length: concurrency }, () => []);
  exampleNames.forEach((name, i) => groups[i % concurrency].push(name));

  const workerPath = fileURLToPath(new URL('./test-e2e-worker.mjs', import.meta.url));

  // Track active workers for cleanup
  /** @type {Set<import('child_process').ChildProcess>} */
  const activeWorkers = new Set();

  const cleanup = () => {
    logger('\nShutting down workers...', { color: 'yellow' });
    for (const w of activeWorkers) {
      if (!w.killed) w.kill('SIGTERM');
    }
    process.exit(1);
  };
  process.on('SIGINT', cleanup);
  process.on('SIGTERM', cleanup);

  const startTime = Date.now();

  /** @type {Map<string, import('../e2e/runner.mjs').TestResult[]>} */
  const allResults = new Map();

  // Launch all workers in parallel
  const workerPromises = groups.map((group, idx) => {
    const workerLogFile = join(logDir, `worker-${idx + 1}.log`);
    const p = runWorker(group, idx, workerPath, workerLogFile, activeWorkers).then((workerResults) => {
      for (const [name, results] of workerResults) {
        allResults.set(name, results);
      }
    });
    return p;
  });

  try {
    await Promise.all(workerPromises);
  } catch (err) {
    logger(`Worker failure: ${err}`, { color: 'red' });
    // Fall through — still print results from workers that completed
  } finally {
    process.removeListener('SIGINT', cleanup);
    process.removeListener('SIGTERM', cleanup);
  }

  await cleanupStaleE2EProcesses({ stage: 'after' });

  const elapsed = ((Date.now() - startTime) / 1000).toFixed(1);
  logger(`\nTotal time: ${elapsed}s`, { color: 'cyan' });
  logger(`Logs saved to: ${logDir}`, { color: 'cyan' });

  printSummary(allResults);

  const totalFailed = [...allResults.values()]
    .flat()
    .filter((r) => !r.passed).length;

  // Close log files synchronously so they flush before exit
  closeLogFiles();
  process.exit(totalFailed > 0 ? 1 : 0);
}

main().catch((err) => {
  logger(`Fatal error: ${err}`, { color: 'red' });
  process.exit(1);
});
