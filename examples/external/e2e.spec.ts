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
        await page.waitForSelector('div#root', { timeout: 10_000 });
        const root = await page.$('#root');
        const innerHTML = await root?.innerHTML();
        expect(innerHTML).toContain('<div>jquery: jquery</div>');
        expect(innerHTML).toContain('<div>react-dom: react-dom</div>');
        expect(innerHTML).toContain('<div>react: react</div');
      },
      command
    );

  await ctx.test('run start', () => runTest());
  await ctx.test('run preview', () => runTest('preview'));
}
