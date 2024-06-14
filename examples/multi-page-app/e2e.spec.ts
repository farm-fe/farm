import { test, expect } from 'vitest';
import { startProjectAndTest } from '../../e2e/vitestSetup';
import { basename, dirname } from 'path';
import { fileURLToPath } from 'url';

const name = basename(import.meta.url);
const projectPath = dirname(fileURLToPath(import.meta.url));

test(`e2e tests - ${name}`, async () => {
  const runTest = (command?: 'start' | 'preview') =>
    startProjectAndTest(
      projectPath,
      async (page) => {
        const host = new URL(page.url()).origin;
        {
          await page.goto(`${host}/about?query=1`)

          const root = await page.$('body');
          const innerHTML = await root?.innerHTML();
          expect(innerHTML).toContain('about page');
        }

        await page.goto(host)

        {
          await page.goto(`${host}/about#/hello/world?hash=2`)

          const root = await page.$('body');
          const innerHTML = await root?.innerHTML();
          expect(innerHTML).toContain('about page');
        }

      },
      command
    );

  await runTest();
  await runTest('preview');
});
