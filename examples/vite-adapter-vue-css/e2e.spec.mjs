import { startAndTest, expect } from '../../e2e/index.mjs';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

const projectPath = dirname(fileURLToPath(import.meta.url));

export default async function (ctx) {
  const runTest = (command) =>
    startAndTest(
      projectPath,
      async (page) => {
        await page.waitForSelector('.box');
        const box = await page.$('.box');
        expect(box).toBeTruthy();
        const color = await box?.evaluate((el) =>
          getComputedStyle(el).getPropertyValue('background-color')
        );
        expect(color).toBe('rgb(255, 0, 0)');
      },
      command
    );

  await ctx.test('run start', () => runTest());
  await ctx.test('run preview', () => runTest('preview'));
}
