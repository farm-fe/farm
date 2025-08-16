import { test, expect } from 'vitest';
import { startProjectAndTest } from '../../e2e/vitestSetup';
import { basename, dirname } from 'path';
import { fileURLToPath } from 'url';
import { describe } from 'vitest';

const name = basename(import.meta.url);
const projectPath = dirname(fileURLToPath(import.meta.url));

describe(`e2e tests vanilla-extract - ${name}`, async () => {
  const runTest = (command?: 'start' | 'preview') =>
    startProjectAndTest(
      projectPath,
      async (page) => {
        await page.waitForSelector('#app');

        const app = await page.$('#app');
        expect(app).toBeTruthy();
        const body = await page.$('body');
        expect(body).toBeTruthy();

        const color = await body?.evaluate((el) => {
          console.log('color', getComputedStyle(el).getPropertyValue('background-color'));
          return getComputedStyle(el).getPropertyValue('background-color');
        });

        expect(color).toBe('rgb(36, 36, 36)');
      },
      command
    );

    test('run start', async () => {
      await runTest();
    })

    test('run preview', async () => {
      await runTest("preview");
    })
});
