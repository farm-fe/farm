import { test, expect, describe } from 'vitest';
import { startProjectAndTest } from '../../e2e/vitestSetup';
import { basename, dirname } from 'path';
import { fileURLToPath } from 'url';

const name = basename(import.meta.url);
const projectPath = dirname(fileURLToPath(import.meta.url));

describe(`e2e tests - ${name}`, async () => {
  const runTest = (command?: 'start' | 'preview') =>
    startProjectAndTest(
      projectPath,
      async (page) => {
        const promise = page.waitForEvent('console', {
          predicate: (msg) => msg.text() === 'antd button clicked'
        });

        const button = await page.waitForSelector('.test-antd-button');

        button.click();
        await promise;
      },
      command
    );

  test(`exmaples ${name} run start`, async () => {
    await runTest();
  });

  test(`exmaples ${name} run prevew`, async () => {
    await runTest('preview');
  });
});
