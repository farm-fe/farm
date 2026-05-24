import { startAndTest, expect } from '../../e2e/index.mjs';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

const projectPath = dirname(fileURLToPath(import.meta.url));

export default async function (ctx) {
  const runTest = (command) =>
    startAndTest(
      projectPath,
      async (page) => {
        await page.waitForSelector('#root > *', { timeout: 10_000 });
        expect(await page.textContent('h1')).toBe('Farm + Vue');
        expect(await page.textContent('.intro')).toContain('@farmfe/plugin-vue');
        expect(await page.textContent('.card strong')).toContain('Pinia count: 0');
        await page.locator('.card button').click();
        expect(await page.textContent('.card strong')).toContain('Pinia count: 1');
        await page.locator('a[href="#/about"]').click();
        await page.waitForSelector('.about', { timeout: 10_000 });
        expect(await page.textContent('.about')).toContain('router navigation');
      },
      command
    );

  await ctx.test('run start', () => runTest());
  await ctx.test('run preview', () => runTest('preview'));
}
