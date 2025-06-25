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

        // should load dynamic component
        await page.waitForSelector('div.farm-container', {
          timeout: 10000
        });
        const container = await page.$('div.farm-container');
        const containerInnerHTML = await container?.innerHTML();
        expect(containerInnerHTML).toContain('React + Farm');
      },
      command
    );

  await runTest();
  await runTest('preview');
});
