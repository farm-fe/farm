import { mkdir } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { startAndTest } from '../../e2e/index.mjs';
import { chromium } from 'playwright-chromium';
import { command } from '../../scripts/performance-report.mjs';
import { verifyBuiltFarm } from '../../scripts/performance-correctness-worker.mjs';
import { verifyBusinessBrowser } from './benchmark/business-correctness.mjs';

const project = dirname(fileURLToPath(import.meta.url));
export default async function (ctx) {
  await ctx.test('Commerce development smoke', () => startAndTest(project, async page => {
    await page.waitForSelector('#root > *', { timeout: 10000 });
  }, 'start'));
  await ctx.test('Commerce production business correctness', async () => {
    // Use the shared owned-process executor, not the performance sampler.
    // No timed rounds, base cohort, benchmark graph size or timing threshold.
    await mkdir(join(project, 'reports'), { recursive: true });
    await command(process.execPath, ['scripts/run.mjs', 'build'], { cwd: project, log: join(project, 'reports/functional-build.log'),
      env: { ...process.env, COMMERCE_SCALE: 'business-smoke', COMMERCE_GROUP: 'rust',
        COMMERCE_CACHE_ID: 'functional', COMMERCE_OUTPUT_ID: 'functional' }, timeoutMs: 180000 });
    const browser = await chromium.launch({ headless: true });
    try {
      await verifyBuiltFarm(join(project, 'dist/benchmark/functional/rust'), browser,
        (page, url) => verifyBusinessBrowser(page, url, { expectedCategories: 4,
          categoryIds: ['category-001', 'category-002', 'category-004'] }));
    } finally { await browser.close(); }
  });
}
