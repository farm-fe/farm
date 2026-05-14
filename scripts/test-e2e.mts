/**
 * Farm E2E test runner — no vitest dependency.
 *
 * Executed with ts-node in ESM mode.
 *
 * CLI:
 *   ts-node --esm scripts/test-e2e.mts
 *   ts-node --esm scripts/test-e2e.mts --example react
 *   ts-node --esm scripts/test-e2e.mts --from arco-pro
 *
 * Each example can have an `e2e.spec.ts` that exports a default function
 * receiving a SpecContext.  Examples without `e2e.spec.ts` but with
 * `index.html` get a default smoke test (checks #root > * exists).
 */
import { existsSync, readdirSync, statSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { chromium, type Browser, type BrowserContext } from 'playwright-chromium';
import {
  initBrowser,
  initBrowserContext,
  setBrowserRecoveryHandler,
  startAndTest
} from '../e2e/farm-runner.ts';
import { SpecRunner, printSummary } from '../e2e/runner.ts';
import { logger } from '../e2e/utils.ts';
import type { TestResult } from '../e2e/runner.ts';

// ---------------------------------------------------------------------------
// CLI argument parsing
// ---------------------------------------------------------------------------

interface CliArgs {
  example: string | undefined;
  startFrom: string | undefined;
}

function parseArgs(argv: string[]): CliArgs {
  let example: string | undefined;
  let startFrom: string | undefined;

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
  }

  return { example, startFrom };
}

// ---------------------------------------------------------------------------
// Example discovery
// ---------------------------------------------------------------------------

/** Examples that are intentionally excluded from the default test sweep. */
const EXCLUDE_FROM_DEFAULT: ReadonlySet<string> = new Set(['issues1433', 'nestjs']);

const EXAMPLES_DIR = resolve(process.cwd(), 'examples');

function discoverExamples(args: CliArgs): string[] {
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
// Default smoke test (for examples without their own e2e.spec.ts)
// ---------------------------------------------------------------------------

async function runDefaultTest(examplePath: string, exampleName: string): Promise<TestResult[]> {
  const results: TestResult[] = [];

  for (const command of ['start', 'preview'] as const) {
    const fullName = `${exampleName} › ${command}`;
    const start = Date.now();
    try {
      await startAndTest(
        examplePath,
        async (page) => {
          if (command === 'start') {
            await page.waitForTimeout(3000);
          } else {
            await page.waitForTimeout(1000);
          }
          await page.waitForSelector('#root > *', { timeout: 10_000 });
          const child = await page.$('#root > *');
          if (!child) throw new Error('#root > * not found');
        },
        command
      );
      const duration = Date.now() - start;
      results.push({ fullName, passed: true, skipped: false, duration });
      logger(`  ✓  ${fullName}  (${duration} ms)`, { title: '', color: 'green' });
    } catch (err) {
      const duration = Date.now() - start;
      const error = err instanceof Error ? err : new Error(String(err));
      results.push({ fullName, passed: false, skipped: false, duration, error });
      logger(`  ✗  ${fullName}  (${duration} ms)\n     ${error.message}`, {
        title: '',
        color: 'red'
      });
    }
  }

  return results;
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main(): Promise<void> {
  const args = parseArgs(process.argv.slice(2));

  if (args.example) {
    logger(`Running E2E for single example: ${args.example}`, { color: 'cyan' });
  } else if (args.startFrom) {
    logger(`Running E2E from example: ${args.startFrom}`, { color: 'cyan' });
  }

  const exampleNames = discoverExamples(args);
  logger(`Discovered ${exampleNames.length} example(s)`, { color: 'cyan' });
  logger('Executing with isolated browser/context per example', {
    color: 'cyan'
  });

  const runner = new SpecRunner();
  const allResults = new Map<string, TestResult[]>();

  for (let exampleIndex = 0; exampleIndex < exampleNames.length; exampleIndex++) {
    const exampleName = exampleNames[exampleIndex];
    const examplePath = join(EXAMPLES_DIR, exampleName);
    const specFile = join(examplePath, 'e2e.spec.ts');
    const hasIndexHtml = existsSync(join(examplePath, 'index.html'));
    const hasSpecFile = existsSync(specFile);
    const shouldRunDefault = hasIndexHtml && !EXCLUDE_FROM_DEFAULT.has(exampleName);

    if (!hasSpecFile && !shouldRunDefault) {
      logger(`Skipping ${exampleName} (no index.html or in exclude list)`, {
        color: 'yellow'
      });
      continue;
    }

    logger(`\nRunning example ${exampleIndex + 1}/${exampleNames.length}: ${exampleName}`, {
      title: 'E2E EXAMPLE',
      color: 'cyan'
    });

    let browser: Browser | null = null;
    let context: BrowserContext | null = null;

    const recreateBrowserAndContext = async (): Promise<void> => {
      if (context) {
        await context.close().catch(() => {});
        context = null;
      }
      if (browser?.isConnected()) {
        await browser.close().catch(() => {});
      }

      browser = await chromium.launch({ headless: true });
      initBrowser(browser);
      context = await browser.newContext();
      initBrowserContext(context);
    };

    try {
      await recreateBrowserAndContext();
      setBrowserRecoveryHandler(async () => {
        await recreateBrowserAndContext();
      });

      if (hasSpecFile) {
        // Run custom spec
        const specUrl = pathToFileURL(specFile).href;
        try {
          const results = await runner.run(specUrl, exampleName);
          allResults.set(exampleName, results);
        } catch (err) {
          logger(
            `Failed to load spec ${exampleName}: ${err}`,
            { color: 'red' }
          );
          allResults.set(exampleName, [
            {
              fullName: `${exampleName} (spec load error)`,
              passed: false,
              skipped: false,
              duration: 0,
              error: err instanceof Error ? err : new Error(String(err))
            }
          ]);
        }
      } else {
        // Run default smoke test
        logger(`\nRunning default smoke test: ${exampleName}`, {
          title: 'E2E SPEC',
          color: 'cyan'
        });
        const results = await runDefaultTest(examplePath, exampleName);
        allResults.set(exampleName, results);
      }
    } finally {
      setBrowserRecoveryHandler(null);
      if (context) {
        await context.close().catch(() => {});
      }
      if (browser?.isConnected()) {
        await browser.close().catch(() => {});
      }
    }
  }

  printSummary(allResults);

  const totalFailed = [...allResults.values()]
    .flat()
    .filter((r) => !r.passed).length;

  process.exit(totalFailed > 0 ? 1 : 0);
}

main().catch((err) => {
  logger(`Fatal error: ${err}`, { color: 'red' });
  process.exit(1);
});
