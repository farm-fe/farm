import { basename, dirname } from 'path';
import { fileURLToPath } from 'url';
import { expect, test } from 'vitest';
import { startProjectAndTest } from '../../e2e/vitestSetup';

const name = basename(import.meta.url);
const projectPath = dirname(fileURLToPath(import.meta.url));

test(`e2e tests - ${name}`, async () => {
  const runTest = (command?: 'start' | 'preview') =>
    startProjectAndTest(
      projectPath,
      async (page) => {
        const consoleIssues: string[] = [];
        const requestIssues: string[] = [];

        page.on('console', (msg) => {
          const text = msg.text();
          if (
            msg.type() === 'error' ||
            text.includes('WASM initialization error:')
          ) {
            consoleIssues.push(`${msg.type()}: ${text}`);
          }
        });

        page.on('requestfailed', (req) => {
          requestIssues.push(`${req.url()} ${req.failure()?.errorText || ''}`);
        });

        await page.waitForSelector('#root > *', { timeout: 10000 });
        await page.waitForSelector('[data-testid="wasm-result"]', {
          timeout: 10000
        });

        const title = await page.textContent('h1');
        expect(title).toContain('Farm + React');

        await page.waitForFunction(() => {
          const el = document.querySelector('[data-testid="wasm-result"]');
          const text = el?.textContent || '';
          return (
            text.length > 0 &&
            !text.includes('Loading WASM result...') &&
            !text.includes('WASM initialization error:')
          );
        });

        const result = await page.textContent('[data-testid="wasm-result"]');
        expect(result).toBeTruthy();
        expect(
          result?.includes('interface Root') ||
            result?.includes('Resolved WASM payload')
        ).toBe(true);

        // Give runtime listeners a short window to capture late errors.
        await page.waitForTimeout(500);

        expect(consoleIssues).toEqual([]);
        expect(requestIssues).toEqual([]);
      },
      command
    );

  await runTest();
  await runTest('preview');
});
