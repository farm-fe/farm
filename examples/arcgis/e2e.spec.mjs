import { startAndTest } from '../../e2e/index.mjs';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

const projectPath = dirname(fileURLToPath(import.meta.url));

export default async function (ctx) {
  const runTest = (command = 'start') =>
    startAndTest(
      projectPath,
      async (page) => {
        await page.waitForSelector('arcgis-map');
        await new Promise((resolve, reject) => {
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
