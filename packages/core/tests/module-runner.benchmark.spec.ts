import { performance } from 'node:perf_hooks';
import { describe, expect, test } from 'vitest';
import { FarmModuleRunner } from '../src/module-runner/runner.js';
import type {
  FetchFunctionOptions,
  ModuleRunnerTransport
} from '../src/module-runner/types.js';

type BenchResult = {
  name: string;
  iterations: number;
  totalMs: number;
  avgMs: number;
  opsPerSec: number;
};

function toBenchResult(
  name: string,
  iterations: number,
  totalMs: number
): BenchResult {
  const avgMs = totalMs / iterations;
  const opsPerSec = iterations / (totalMs / 1000);
  return {
    name,
    iterations,
    totalMs: Number(totalMs.toFixed(3)),
    avgMs: Number(avgMs.toFixed(4)),
    opsPerSec: Number(opsPerSec.toFixed(2))
  };
}

async function measure(
  name: string,
  iterations: number,
  fn: () => Promise<void>
): Promise<BenchResult> {
  const start = performance.now();
  for (let i = 0; i < iterations; i++) {
    await fn();
  }
  const end = performance.now();
  return toBenchResult(name, iterations, end - start);
}

describe('farm module runner benchmark', () => {
  test.runIf(Boolean(process.env.FARM_BENCH))(
    'captures module-runner baseline metrics',
    async () => {
      let fetchCount = 0;
      const transport: ModuleRunnerTransport = {
        async invoke(name, data) {
          expect(name).toBe('fetchModule');
          const [id, _importer, options] = data as [
            string,
            string | undefined,
            FetchFunctionOptions | undefined
          ];
          fetchCount++;

          if (options?.cached) {
            return { cache: true };
          }

          if (id === '/entry.mjs') {
            return {
              code: "__farm_ssr_export_name__('value', () => 1);",
              file: '/entry.mjs',
              id: '/entry.mjs',
              url: '/entry.mjs',
              invalidate: false,
              map: null
            };
          }

          return {
            code: `__farm_ssr_export_name__('value', () => ${JSON.stringify(id)});`,
            file: id,
            id,
            url: id,
            invalidate: false,
            map: null
          };
        }
      };

      const runner = new FarmModuleRunner({
        transport,
        hmr: false,
        cachePolicy: {
          maxEntries: 50,
          gcSweepPerCycle: 16
        }
      });

      try {
        const cold = await measure('cold_import_unique', 200, async () => {
          const id = `/cold-${Math.random().toString(36).slice(2, 12)}.mjs`;
          await runner.import(id);
        });

        await runner.import('/entry.mjs');
        const warm = await measure('warm_import_same', 2000, async () => {
          await runner.import('/entry.mjs');
        });

        let dedupeFetchCount = 0;
        const dedupeTransport: ModuleRunnerTransport = {
          async invoke(name, data) {
            expect(name).toBe('fetchModule');
            const [id, _importer, options] = data as [
              string,
              string | undefined,
              FetchFunctionOptions | undefined
            ];

            if (id !== '/dedupe.mjs') {
              throw new Error(`unexpected id: ${String(id)}`);
            }

            if (options?.cached) {
              return { cache: true };
            }

            dedupeFetchCount++;
            await new Promise((resolve) => setTimeout(resolve, 1));
            return {
              code: "__farm_ssr_export_name__('value', () => 1);",
              file: '/dedupe.mjs',
              id: '/dedupe.mjs',
              url: '/dedupe.mjs',
              invalidate: false,
              map: null
            };
          }
        };
        const dedupeRunner = new FarmModuleRunner({
          transport: dedupeTransport,
          hmr: false
        });

        const dedupe = await measure('concurrent_dedupe_20x', 100, async () => {
          await Promise.all(
            Array.from({ length: 20 }, () => dedupeRunner.import('/dedupe.mjs'))
          );
          dedupeRunner.clearCache();
        });

        expect(dedupeFetchCount).toBe(100);
        await dedupeRunner.close();

        const gcChurn = await measure('gc_churn_200_unique', 200, async () => {
          const id = `/gc-${Math.random().toString(36).slice(2, 12)}.mjs`;
          await runner.import(id);
        });

        const results = [cold, warm, dedupe, gcChurn];
        console.log(
          '[module-runner-bench]',
          JSON.stringify(
            {
              timestamp: new Date().toISOString(),
              node: process.version,
              fetchCount,
              results
            },
            null,
            2
          )
        );
      } finally {
        await runner.close();
      }
    }
  );
});
