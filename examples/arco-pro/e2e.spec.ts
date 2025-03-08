import { test, expect, describe } from 'vitest';
import { startProjectAndTest } from '../../e2e/vitestSetup';
import path, { basename, dirname } from 'path';
import { fileURLToPath } from 'url';
import { readFileSync, writeFileSync } from 'fs';

const name = basename(import.meta.url);
const projectPath = dirname(fileURLToPath(import.meta.url));

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

describe(`e2e tests - ${name}`, async () => {
  const runTest = (command: 'start' | 'preview' = 'start') =>
    startProjectAndTest(
      projectPath,
      async (page) => {
        await page.waitForSelector('button.arco-btn.arco-btn-primary', {
          timeout: 10000
        });
        const root = await page.$('#root');
        const innerHTML = await root?.innerHTML();
        expect(innerHTML).toContain('<span>register account</span');
        expect(innerHTML).toContain('<span>login</span');

        const loginButton = await page.$('button.arco-btn.arco-btn-primary');
        expect(loginButton).toBeTruthy();
        await loginButton?.click();
        console.log('click login button');
        // shoud navigate to dashboard
        await page.waitForSelector(
          'a[href="https://arco.design/react/docs/start"]',
          { timeout: 10000 }
        );
        const reactLink = await page.$(
          'a[href="https://arco.design/react/docs/start"]'
        );
        expect(reactLink).toBeTruthy();

        // browser HMR should work
        if (command === 'start') {
          const filePath = path.join(projectPath, 'src', 'pages', 'dashboard', 'workplace', 'docs.tsx');
          const content = readFileSync(filePath, 'utf-8');
          writeFileSync(
            filePath,
            content.replace(
              'https://arco.design/react/docs/start',
              'https://arco.design/react/docs/start/farm'
            )
          );
          const reactLinkHmr = await page.waitForSelector(
            'a[href="https://arco.design/react/docs/start/farm"]'
          );
          expect(reactLinkHmr).toBeTruthy();

          await delay(3000);

          // revert change
          writeFileSync(filePath, content);
          const reactLinkHmr2 = await page.waitForSelector(
            'a[href="https://arco.design/react/docs/start"]'
          );
          expect(reactLinkHmr2).toBeTruthy();
        }
      },
      command
    );

  test('exmaples arco-pro run start', async () => {
    await runTest();
  })

  test('exampels arco-pro run preview', async () => {
    await runTest('preview');
  })
});
