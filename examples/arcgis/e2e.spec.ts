import { startAndTest } from '../../e2e/index.ts';
import type { SpecContext } from '../../e2e/index.ts';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

const projectPath = dirname(fileURLToPath(import.meta.url));

export default async function (ctx: SpecContext): Promise<void> {
  const runTest = (command: 'start' | 'preview' = 'start') =>
    startAndTest(
      projectPath,
      async (page) => {
        await page.waitForSelector('arcgis-map');
        await new Promise<void>((resolve, reject) => {
          page.on('console', (msg) => {
            if (msg.text().includes('arcgis all ready')) resolve();
          });
        });
      },
      command
    );

  await ctx.test('run start', () => runTest());
  await ctx.test('run preview', () => runTest('preview'));
}
