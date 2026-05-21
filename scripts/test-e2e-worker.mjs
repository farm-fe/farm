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
import { existsSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
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

  for (const command of commands) {
    const fullName = `${exampleName} › ${command}`;
    const start = Date.now();

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
          process.send({ type: 'results', example: name, results });
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
