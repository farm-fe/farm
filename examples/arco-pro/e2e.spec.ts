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
      },
      command
    );

  await runTest();
  await runTest('preview');
});
