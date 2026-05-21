import { startAndTest } from '../../e2e/index.mjs';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

const projectPath = dirname(fileURLToPath(import.meta.url));

export default async function (ctx) {
  const runTest = (command) =>
    startAndTest(
      projectPath,
      async (page) => {
        await page.waitForSelector('body > *', { timeout: 10_000 });
      },
      command
    );

  await ctx.test('run start', () => runTest());
  await ctx.test('run preview', () => runTest('preview'));
}
