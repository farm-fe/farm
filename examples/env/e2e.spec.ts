import { startAndTest } from '../../e2e/index.ts';
import type { SpecContext } from '../../e2e/index.ts';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

const projectPath = dirname(fileURLToPath(import.meta.url));

export default async function (ctx: SpecContext): Promise<void> {
  const runTest = (command?: 'start' | 'preview') =>
    startAndTest(
      projectPath,
      async (page) => {
        // Just verify the page renders something in the body
        await page.waitForSelector('body > *', { timeout: 10_000 });
      },
      command
    );

  await ctx.test('run start', () => runTest());
  await ctx.test('run preview', () => runTest('preview'));
}
