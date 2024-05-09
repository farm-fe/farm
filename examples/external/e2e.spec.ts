import { test, expect } from 'vitest';
import { startProjectAndTest } from '../../e2e/vitestSetup';
import path, { basename, dirname } from 'path';
import { fileURLToPath } from 'url';
import { readFileSync, writeFileSync } from 'fs';

const name = basename(import.meta.url);
const projectPath = dirname(fileURLToPath(import.meta.url));

test(`e2e tests - ${name}`, async () => {
  const runTest = (command?: 'start' | 'preview') =>
    startProjectAndTest(
      projectPath,
      async (page) => {
        console.log(page.url());
        await page.waitForSelector('div#root', {
          timeout: 10000
        });
        const root = await page.$('#root');
        const innerHTML = await root?.innerHTML();
        expect(innerHTML).toContain('<div>jquery: jquery</div>');
        expect(innerHTML).toContain('<div>react-dom: react-dom</div>');
        expect(innerHTML).toContain('<div>react: react</div');
      },
      command
    );

  await runTest();
  await runTest('preview');
});
