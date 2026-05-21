import { startAndTest } from '../../e2e/index.mjs';
import { dirname } from 'path';
import { fileURLToPath } from 'url';
import { execa } from 'execa';

const projectPath = dirname(fileURLToPath(import.meta.url));

export default async function (ctx) {
  await execa('npm', ['run', 'build'], { cwd: projectPath });

  const runTest = (command) =>
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
