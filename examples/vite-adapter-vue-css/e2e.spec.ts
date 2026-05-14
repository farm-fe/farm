import { startAndTest, expect } from '../../e2e/index.ts';
import type { SpecContext } from '../../e2e/index.ts';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

const projectPath = dirname(fileURLToPath(import.meta.url));

export default async function (ctx: SpecContext): Promise<void> {
  const runTest = (command?: 'start' | 'preview') =>
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
