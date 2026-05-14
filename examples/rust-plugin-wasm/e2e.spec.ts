import { startAndTest, expect } from '../../e2e/index.ts';
import type { SpecContext } from '../../e2e/index.ts';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

const projectPath = dirname(fileURLToPath(import.meta.url));

export default async function (ctx: SpecContext): Promise<void> {
  const runTest = (command?: 'start' | 'preview') =>
    startAndTest(
      projectPath,
      async (page) => {
        const consoleIssues: string[] = [];
        const requestIssues: string[] = [];

        page.on('console', (msg) => {
          const text = msg.text();
          if (msg.type() === 'error' || text.includes('WASM initialization error:')) {
            consoleIssues.push(`${msg.type()}: ${text}`);
          }
        });

        page.on('requestfailed', (req) => {
          requestIssues.push(`${req.url()} ${req.failure()?.errorText || ''}`);
        });

        await page.waitForSelector('#root > *', { timeout: 10_000 });
        await page.waitForSelector('[data-testid="wasm-result"]', { timeout: 10_000 });

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
          result?.includes('interface Root') || result?.includes('Resolved WASM payload')
        ).toBe(true);

        await page.waitForTimeout(500);

        expect(consoleIssues).toEqual([]);
        expect(requestIssues).toEqual([]);
      },
      command
    );

  await ctx.test('run start', () => runTest());
  await ctx.test('run preview', () => runTest('preview'));
}
