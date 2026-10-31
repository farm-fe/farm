import { startAndTest } from '../../e2e/index.mjs';
import assert from 'node:assert/strict';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

const projectPath = dirname(fileURLToPath(import.meta.url));

export default async function (ctx) {
  const runTest = (command) =>
    startAndTest(
      projectPath,
      async (page) => {
        const value = command === 'preview' ? '2' : '1';
        const file = command === 'preview' ? '.env.production' : '.env.development';
        await page.waitForFunction(
          (expected) => document.querySelector('.custom')?.textContent === expected,
          value,
          { timeout: 10_000 }
        );
        for (const selector of ['.custom', '.prefix', '.globalThis', '.global']) {
          assert.equal(await page.locator(selector).textContent(), value);
        }
        assert.equal(await page.locator('.file').textContent(), file);
        assert.equal(await page.locator('.bool').textContent(), 'true');
        assert.equal(await page.evaluate(() => typeof process), 'undefined');
      },
      command
    );

  await ctx.test('run start', () => runTest());
  await ctx.test('run preview', () => runTest('preview'));
}
