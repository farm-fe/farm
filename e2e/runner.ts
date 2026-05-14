/**
 * Lightweight test-runner framework for Farm's standalone E2E tests.
 *
 * Each spec file exports a `default` async function that receives a
 * `SpecContext`.  Tests run sequentially in the order they are called.
 *
 * Usage in a spec file:
 *
 *   import type { SpecContext } from '../../e2e/runner.ts';
 *
 *   export default async function (ctx: SpecContext): Promise<void> {
 *     await ctx.test('run start', async () => { ... });
 *     await ctx.describe('group', async (g) => {
 *       await g.test('sub-test', async () => { ... });
 *     });
 *   }
 */
import { logger } from './utils.ts';

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

export interface TestResult {
  fullName: string;
  passed: boolean;
  skipped: boolean;
  duration: number;
  error?: Error;
}

/** Passed to every spec's default export. */
export interface SpecContext {
  /** Run a single named test case. */
  test(name: string, fn: () => Promise<void>): Promise<void>;
  /** Group related tests under a named suite. */
  describe(
    suiteName: string,
    fn: (ctx: SpecContext) => Promise<void>
  ): Promise<void>;
  /** Mark a test as skipped (no-op at runtime). */
  skip(name: string): void;
}

/** Shape every spec file's default export must conform to. */
export type SpecFn = (ctx: SpecContext) => Promise<void>;

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

export class SpecRunner {
  private results: TestResult[] = [];

  /**
   * Import and execute a spec file.
   * Returns the list of test results for that spec.
   */
  async run(specPath: string, specLabel: string): Promise<TestResult[]> {
    this.results = [];
    logger(`\nRunning spec: ${specLabel}`, { title: 'E2E SPEC', color: 'cyan' });

    const mod = await import(specPath) as { default?: SpecFn };
    const specFn = mod.default;

    if (typeof specFn !== 'function') {
      throw new Error(
        `Spec file "${specPath}" does not export a default function. ` +
          'Each spec must export: export default async function(ctx: SpecContext) { ... }'
      );
    }

    const ctx = this.buildContext('');
    await specFn(ctx);
    return [...this.results];
  }

  private buildContext(prefix: string): SpecContext {
    const self = this;

    return {
      async test(name, fn) {
        const fullName = prefix ? `${prefix} › ${name}` : name;
        const start = Date.now();
        try {
          await fn();
          const duration = Date.now() - start;
          self.results.push({ fullName, passed: true, skipped: false, duration });
          logger(`  ✓  ${fullName}  (${duration} ms)`, { title: '', color: 'green' });
        } catch (err) {
          const duration = Date.now() - start;
          const error = err instanceof Error ? err : new Error(String(err));
          self.results.push({ fullName, passed: false, skipped: false, duration, error });
          logger(`  ✗  ${fullName}  (${duration} ms)\n     ${error.message}`, {
            title: '',
            color: 'red'
          });
        }
      },

      async describe(suiteName, fn) {
        const fullPrefix = prefix ? `${prefix} › ${suiteName}` : suiteName;
        logger(`\n  ⬛  ${fullPrefix}`, { title: '', color: 'cyan' });
        await fn(self.buildContext(fullPrefix));
      },

      skip(name) {
        const fullName = prefix ? `${prefix} › ${name}` : name;
        self.results.push({ fullName, passed: true, skipped: true, duration: 0 });
        logger(`  ⊘  ${fullName}  (skipped)`, { title: '', color: 'yellow' });
      }
    };
  }
}

// ---------------------------------------------------------------------------
// Aggregate reporting helper
// ---------------------------------------------------------------------------

export function printSummary(allResults: Map<string, TestResult[]>): void {
  let totalPassed = 0;
  let totalFailed = 0;
  let totalSkipped = 0;

  const failedSpecs: { spec: string; result: TestResult }[] = [];

  for (const [spec, results] of allResults) {
    const passed = results.filter((r) => r.passed && !r.skipped).length;
    const failed = results.filter((r) => !r.passed).length;
    const skipped = results.filter((r) => r.skipped).length;

    totalPassed += passed;
    totalFailed += failed;
    totalSkipped += skipped;

    for (const r of results.filter((r) => !r.passed)) {
      failedSpecs.push({ spec, result: r });
    }
  }

  logger(
    '\n─────────────────────────────────────────────────────────',
    { title: '', color: 'cyan' }
  );
  logger(
    `E2E summary:  ${totalPassed} passed  |  ${totalFailed} failed  |  ${totalSkipped} skipped`,
    { title: 'RESULT', color: totalFailed > 0 ? 'red' : 'green' }
  );

  if (failedSpecs.length > 0) {
    logger('\nFailed tests:', { title: '', color: 'red' });
    for (const { spec, result } of failedSpecs) {
      logger(`  [${spec}]  ${result.fullName}`, { title: '', color: 'red' });
      if (result.error) {
        logger(`    ${result.error.message}`, { title: '', color: 'red' });
      }
    }
  }
}
