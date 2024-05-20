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
        await page.waitForSelector('.box');

        const box = await page.$('.box');
        expect(box).toBeTruthy();

        const color = await box?.evaluate((el) => {
          console.log('color', getComputedStyle(el).getPropertyValue('background-color'));
          return getComputedStyle(el).getPropertyValue('background-color');
        });

        expect(color).toBe('rgb(255, 0, 0)');
      },
      command
    );

  await runTest();
  await runTest('preview');
});
