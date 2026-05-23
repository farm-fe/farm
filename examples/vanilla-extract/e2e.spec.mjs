import { startAndTest, expect } from '../../e2e/index.mjs';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

const projectPath = dirname(fileURLToPath(import.meta.url));

export default async function (ctx) {
  const runTest = (command) =>
    startAndTest(
      projectPath,
      async (page) => {
        await page.waitForSelector('#app');
        const app = await page.$('#app');
        expect(app).toBeTruthy();
        const body = await page.$('body');
        expect(body).toBeTruthy();
        const color = await body?.evaluate((el) =>
          getComputedStyle(el).getPropertyValue('background-color')
        );
        expect(color).toBe('rgb(36, 36, 36)');
      },
      command
    );

  await ctx.test('run start', () => runTest());
  await ctx.test('run preview', () => runTest('preview'));
}
