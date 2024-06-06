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
        await page.waitForSelector('div.public-script', {
          timeout: 10000
        });
        const root = await page.$('div.public-script');
        const innerHTML = await root?.innerHTML();
        expect(innerHTML).toContain('public script');
      },
      command
    );

  await runTest();
  await runTest('preview');
});
