import { logger } from './utils.mjs';

/**
 * @typedef {{
 *   fullName: string;
 *   passed: boolean;
 *   skipped: boolean;
 *   duration: number;
 *   error?: Error;
 * }} TestResult
 */

/**
 * @typedef {{
 *   test(name: string, fn: () => Promise<void>): Promise<void>;
 *   describe(suiteName: string, fn: (ctx: SpecContext) => Promise<void>): Promise<void>;
 *   skip(name: string): void;
 * }} SpecContext
 */

/**
 * @typedef {(ctx: SpecContext) => Promise<void>} SpecFn
 */

export class SpecRunner {
  /** @type {TestResult[]} */
  results = [];

  /**
   * @param {string} specPath
   * @param {string} specLabel
   * @returns {Promise<TestResult[]>}
   */
  async run(specPath, specLabel) {
    this.results = [];
    logger(`\nRunning spec: ${specLabel}`, { title: 'E2E SPEC', color: 'cyan' });

    /** @type {{ default?: SpecFn }} */
    const mod = await import(specPath);
    const specFn = mod.default;

    if (typeof specFn !== 'function') {
      throw new Error(
        `Spec file "${specPath}" does not export a default function. ` +
          'Each spec must export: export default async function(ctx) { ... }'
      );
    }

    const ctx = this.#buildContext('');
    await specFn(ctx);
    return [...this.results];
  }

  /**
   * @param {string} prefix
   * @returns {SpecContext}
   */
  #buildContext(prefix) {
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
        await fn(self.#buildContext(fullPrefix));
      },

      skip(name) {
        const fullName = prefix ? `${prefix} › ${name}` : name;
        self.results.push({ fullName, passed: true, skipped: true, duration: 0 });
        logger(`  ⊘  ${fullName}  (skipped)`, { title: '', color: 'yellow' });
      }
    };
  }
}

/**
 * @param {Map<string, TestResult[]>} allResults
 */
export function printSummary(allResults) {
  let totalPassed = 0;
  let totalFailed = 0;
  let totalSkipped = 0;

  const failedSpecs = /** @type {{ spec: string; result: TestResult }[]} */ ([]);

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
