import { startAndTest, expect } from '../../e2e/index.mjs';
import path, { dirname } from 'path';
import { fileURLToPath } from 'url';
import { readFileSync, writeFileSync } from 'fs';

const projectPath = dirname(fileURLToPath(import.meta.url));

const delay = (ms) => new Promise((r) => setTimeout(r, ms));

export default async function (ctx) {
  const runTest = (command = 'start') =>
    startAndTest(
      projectPath,
      async (page) => {
        await page.waitForSelector('button.arco-btn.arco-btn-primary', { timeout: 10_000 });
        const root = await page.$('#root');
        const innerHTML = await root?.innerHTML();
        expect(innerHTML).toContain('<span>register account</span');
        expect(innerHTML).toContain('<span>login</span');

        const loginButton = await page.$('button.arco-btn.arco-btn-primary');
        expect(loginButton).toBeTruthy();
        await loginButton?.click();

        await page.waitForSelector(
          'a[href="https://arco.design/react/docs/start"]',
          { timeout: 10_000 }
        );
        const reactLink = await page.$(
          'a[href="https://arco.design/react/docs/start"]'
        );
        expect(reactLink).toBeTruthy();

        if (command === 'start') {
          const filePath = path.join(
            projectPath,
            'src',
            'pages',
            'dashboard',
            'workplace',
            'docs.tsx'
          );
          const content = readFileSync(filePath, 'utf-8');
          writeFileSync(
            filePath,
            content.replace(
              'https://arco.design/react/docs/start',
              'https://arco.design/react/docs/start/farm'
            )
          );

          try {
            const reactLinkHmr = await page.waitForSelector(
              'a[href="https://arco.design/react/docs/start/farm"]'
            );
            expect(reactLinkHmr).toBeTruthy();

            await delay(3000);

            writeFileSync(filePath, content);
            const reactLinkHmr2 = await page.waitForSelector(
              'a[href="https://arco.design/react/docs/start"]'
            );
            expect(reactLinkHmr2).toBeTruthy();
          } catch (e) {
            writeFileSync(filePath, content); // always restore
            throw e;
          }
        }
      },
      command
    );

  await ctx.test('run start', () => runTest('start'));
  await ctx.test('run preview', () => runTest('preview'));
}
