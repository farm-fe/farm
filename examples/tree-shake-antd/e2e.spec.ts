import { startAndTest } from '../../e2e/index.ts';
import type { SpecContext } from '../../e2e/index.ts';
import { dirname } from 'path';
import { fileURLToPath } from 'url';
import { execa } from 'execa';

const projectPath = dirname(fileURLToPath(import.meta.url));

export default async function (ctx: SpecContext): Promise<void> {
  // Build first before running tests
  await execa('npm', ['run', 'build'], { cwd: projectPath });

  const runTest = (command?: 'start' | 'preview') =>
    startAndTest(
      projectPath,
      async (page) => {
        const promise = page.waitForEvent('console', {
          predicate: (msg) => msg.text() === 'antd button clicked'
        });
        const button = await page.waitForSelector('.test-antd-button');
        await button.click();
        await promise;
      },
      command
    );

  await ctx.test('run start', () => runTest());
  await ctx.test('run preview', () => runTest('preview'));
}
