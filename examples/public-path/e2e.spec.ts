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
        await page.waitForSelector('div.public-script', { timeout: 10_000 });
        const root = await page.$('div.public-script');
        const innerHTML = await root?.innerHTML();
        expect(innerHTML).toContain('public script');

        await page.waitForSelector('div.farm-container', { timeout: 10_000 });
        const container = await page.$('div.farm-container');
        const containerInnerHTML = await container?.innerHTML();
        expect(containerInnerHTML).toContain('React + Farm');
      },
      command
    );

  await ctx.test('run start', () => runTest());
  await ctx.test('run preview', () => runTest('preview'));
}
