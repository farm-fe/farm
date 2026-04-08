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
        if (command === 'start') {
          await page.waitForTimeout(3000);
        } else {
          await page.waitForTimeout(1000);
        }

        await page.waitForSelector('#root > *', { timeout: 10000 });
        const child = await page.$('#root > *');
        expect(child).toBeTruthy();

        const heading = await page.$('h1');
        expect(heading).toBeTruthy();
        const headingText = await heading?.textContent();
        expect(headingText).toContain('TailwindCSS Rust Plugin');
      },
      command
    );

  test(`example ${name} run start`, async () => {
    await runTest();
  });

  test(`example ${name} run preview`, async () => {
    await runTest('preview');
  });
});
