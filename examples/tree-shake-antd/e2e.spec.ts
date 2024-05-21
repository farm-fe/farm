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
        const button = await page.waitForSelector('.test-antd-button');

        const promise = page.waitForEvent('console', {
          predicate: (msg) => msg.text() === 'antd button clicked'
        });

        button.click();
        await promise;
      },
      command
    );

  await runTest();
  await runTest('preview');
});
