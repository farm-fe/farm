/**
 * Farm E2E test worker.
 *
 * Forked by the orchestrator (scripts/test-e2e.mjs).
 * Each worker:
 *   - Launches ONE browser at startup (kept alive across examples)
 *   - Runs a subset of examples sequentially
 *   - Creates a fresh context per example
 *   - Reports results back to the orchestrator via IPC
 */
import { existsSync, readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { execa } from 'execa';
import { chromium } from 'playwright-chromium';
import {
  initBrowser,
  initBrowserContext,
  setBrowserRecoveryHandler,
  startAndTest
} from '../e2e/farm-runner.mjs';
import { SpecRunner } from '../e2e/runner.mjs';
import { logger, setLogFile, closeLogFiles } from '../e2e/utils.mjs';

const EXAMPLES_DIR = resolve(process.cwd(), 'examples');
const EXCLUDE_FROM_DEFAULT = new Set(['issues1433', 'nestjs']);

/**
 * Some examples opt into `server.writeToDisk: true` in their farm config,
 * which makes `farm start` (dev mode) overwrite the production `dist/`
 * artifacts with dev-mode bundles containing a hard-coded HMR port. When
 * `start` and `preview` run sequentially, the subsequent `farm preview`
 * would otherwise serve those stale dev artifacts and the browser's HMR
 * client would try to connect to a closed dev-server port, failing the
 * test. Detect this case by scanning the farm config for `writeToDisk`
 * set truthy, so we can rebuild `dist/` before running `preview`.
 *
 * @param {string} examplePath
 * @returns {boolean}
 */
function exampleWritesToDiskInDev(examplePath) {
  for (const file of ['farm.config.ts', 'farm.config.js', 'farm.config.mjs', 'farm.config.cjs']) {
    const p = join(examplePath, file);
    if (!existsSync(p)) continue;
    try {
      const src = readFileSync(p, 'utf8');
      // Match writeToDisk: true (allowing whitespace). Strings/false are ignored.
      if (/writeToDisk\s*:\s*true\b/.test(src)) return true;
    } catch {
      // ignore — treat as not writing to disk
    }
  }
  return false;
}

/**
 * Rebuild the example's `dist/` so that the subsequent `preview` command
 * serves a fresh production bundle (used when `start` may have polluted
 * `dist/` via `server.writeToDisk: true`).
 *
 * @param {string} examplePath
 * @param {string} exampleName
 * @returns {Promise<void>}
 */
async function rebuildExampleDist(examplePath, exampleName) {
  logger(`Rebuilding dist for ${exampleName} before preview...`, { color: 'cyan' });
  await execa('npm', ['run', 'build'], {
    cwd: examplePath,
    timeout: 180_000,
    env: { ...process.env, BROWSER: 'none', NO_COLOR: 'true' }
  });
}

// ---------------------------------------------------------------------------
// Default smoke test (start + preview in parallel)
// ---------------------------------------------------------------------------

/**
 * @param {string} examplePath
 * @param {string} exampleName
 * @returns {Promise<import('../e2e/runner.mjs').TestResult[]>}
 */
async function runDefaultTest(examplePath, exampleName) {
  const commands = /** @type {const} */ (['start', 'preview']);
  /** @type {import('../e2e/runner.mjs').TestResult[]} */
  const results = [];

  // If `start` (dev mode) might overwrite `dist/` with stale HMR-bearing
  // artifacts, plan a rebuild step before running `preview`.
  const needsRebuildBeforePreview = exampleWritesToDiskInDev(examplePath);

  for (const command of commands) {
    const fullName = `${exampleName} › ${command}`;
    const start = Date.now();

    if (command === 'preview' && needsRebuildBeforePreview) {
      try {
        await rebuildExampleDist(examplePath, exampleName);
      } catch (reason) {
        const error = reason instanceof Error ? reason : new Error(String(reason ?? 'Unknown error'));
        logger(`  ✗  ${fullName}  —  rebuild failed: ${error.message}`, { title: '', color: 'red' });
        results.push({
          fullName,
          passed: false,
          skipped: false,
          duration: Date.now() - start,
          error
        });
        continue;
      }
    }

    try {
      await startAndTest(
        examplePath,
        async (page) => {
          const delay = command === 'start' ? 3000 : 1000;
          await page.waitForTimeout(delay);
          await page.waitForSelector('#root > *', { timeout: 10_000 });
          const child = await page.$('#root > *');
          if (!child) throw new Error('#root > * not found');
        },
        command
      );

      const duration = Date.now() - start;
      logger(`  ✓  ${fullName}  (${duration} ms)`, { title: '', color: 'green', level: 'progress' });
      results.push({ fullName, passed: true, skipped: false, duration });
    } catch (reason) {
      const error = reason instanceof Error ? reason : new Error(String(reason ?? 'Unknown error'));
      logger(`  ✗  ${fullName}  —  ${error.message}`, { title: '', color: 'red' });
      results.push({
        fullName,
        passed: false,
        skipped: false,
        duration: Date.now() - start,
        error
      });
    }
  }

  return results;
}

// ---------------------------------------------------------------------------
// Run a single example
// ---------------------------------------------------------------------------

/**
 * @param {string} exampleName
 * @returns {Promise<{ name: string, results: import('../e2e/runner.mjs').TestResult[] }>}
 */
async function runExample(exampleName) {
  const examplePath = join(EXAMPLES_DIR, exampleName);
  const specFile = join(examplePath, 'e2e.spec.mjs');
  const hasIndexHtml = existsSync(join(examplePath, 'index.html'));
  const hasSpecFile = existsSync(specFile);
  const shouldRunDefault = hasIndexHtml && !EXCLUDE_FROM_DEFAULT.has(exampleName);

  if (!hasSpecFile && !shouldRunDefault) {
    return { name: exampleName, results: [] };
  }

  logger(`\nRunning example: ${exampleName}`, {
    title: 'E2E EXAMPLE',
    color: 'cyan'
  });

  // Module-level singletons in farm-runner.mjs are already initialized
  // by runWorker (initBrowser/initBrowserContext). The spec uses them implicitly.
  if (hasSpecFile) {
    const runner = new SpecRunner();
    const specUrl = pathToFileURL(specFile).href;
    try {
      const results = await runner.run(specUrl, exampleName);
      return { name: exampleName, results };
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      return {
        name: exampleName,
        results: [{
          fullName: `${exampleName} (spec load error)`,
          passed: false,
          skipped: false,
          duration: 0,
          error
        }]
      };
    }
  } else {
    const results = await runDefaultTest(examplePath, exampleName);
    return { name: exampleName, results };
  }
}

// ---------------------------------------------------------------------------
// Worker main — receive examples via IPC, run them, report results
// ---------------------------------------------------------------------------

/**
 * @param {string[]} exampleNames
 */
async function runWorker(exampleNames) {
  const ciArgs = process.env.CI
    ? ['--disable-dev-shm-usage', '--no-sandbox', '--disable-gpu']
    : [];

  /** @type {import('playwright-chromium').Browser | null} */
  let browser = null;

  try {
    browser = await chromium.launch({ headless: true, args: ciArgs });
    initBrowser(browser);
    logger(`Worker started — ${exampleNames.length} example(s) assigned`, {
      color: 'cyan'
    });

    for (const exampleName of exampleNames) {
      let context = null;
      try {
        context = await browser.newContext();
        initBrowserContext(context);

        setBrowserRecoveryHandler(async () => {
          // Context-level recovery: close old context, create new one
          if (context) {
            await context.close().catch(() => {});
            context = null;
          }
          if (!browser?.isConnected()) {
            throw new Error('Browser disconnected, cannot recover context');
          }
          context = await browser.newContext();
          initBrowserContext(context);
        });

        const { name, results } = await runExample(exampleName);
        if (results.length > 0 && process.send) {
          // Serialize results so they survive IPC structured cloning —
          // in particular, Error instances do not survive `fork()` IPC
          // and would otherwise be received as `{}` on the orchestrator,
          // causing the summary to print `undefined` for failure
          // messages.
          const wireResults = results.map((r) => ({
            fullName: r.fullName,
            passed: r.passed,
            skipped: r.skipped,
            duration: r.duration,
            errorMessage: r.error
              ? (r.error instanceof Error ? r.error.message : String(r.error))
              : undefined,
            errorStack:
              r.error && r.error instanceof Error && r.error.stack
                ? r.error.stack
                : undefined
          }));
          process.send({ type: 'results', example: name, results: wireResults });
        } else if (process.send) {
          process.send({ type: 'skip', example: name });
        }
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        logger(`Example ${exampleName} failed: ${error.message}`, { color: 'red' });
        if (process.send) {
          process.send({
            type: 'error',
            example: exampleName,
            error: error.message
          });
        }
      } finally {
        setBrowserRecoveryHandler(null);
        if (context) {
          await context.close().catch(() => {});
          context = null;
        }
        initBrowserContext(null);
      }
    }
  } finally {
    if (browser?.isConnected()) {
      await browser.close().catch(() => {});
    }
    if (process.send) {
      process.send({ type: 'done' });
    }
  }
}

process.on('message', (msg) => {
  if (msg?.type === 'run') {
    if (msg.logFile) {
      setLogFile(msg.logFile);
    }
    runWorker(msg.examples)
      .then(() => closeLogFiles())
      .catch((err) => {
        if (process.send) {
          process.send({ type: 'fatal', error: err instanceof Error ? err.message : String(err) });
        }
        process.exitCode = 1;
        closeLogFiles();
      });
  }
});
