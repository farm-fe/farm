import { startAndTest, expect } from '../../e2e/index.mjs';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

const projectPath = dirname(fileURLToPath(import.meta.url));

export default async function (ctx) {
  const runTest = (command) =>
    startAndTest(
      projectPath,
      async (page) => {
        const host = new URL(page.url()).origin;
        {
          await page.goto(`${host}/about?query=1`);
          const root = await page.$('body');
          const innerHTML = await root?.innerHTML();
          expect(innerHTML).toContain('about page');
        }

        await page.goto(host);

        {
          await page.goto(`${host}/about#/hello/world?hash=2`);
          const root = await page.$('body');
          const innerHTML = await root?.innerHTML();
          expect(innerHTML).toContain('about page');
        }
      },
      command
    );

  await ctx.test('run start', () => runTest());
  await ctx.test('run preview', () => runTest('preview'));
}
