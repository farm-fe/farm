import { test, describe } from 'vitest';
import { startProjectAndTest } from '../../e2e/vitestSetup';
import { basename, dirname } from 'path';
import { fileURLToPath } from 'url';

const name = basename(import.meta.url);
const projectPath = dirname(fileURLToPath(import.meta.url));

describe(`e2e tests - ${name}`, async () => {
  const runTest = (command: 'start' | 'preview' = 'start') =>
    startProjectAndTest(
      projectPath,
      async (page) => {
        await page.waitForSelector('arcgis-map');

        return new Promise((resolve, reject) => {
          page.on('console', (msg) => {
            if (msg.type() === 'error') {
              reject(msg.text());
              return;
            }

            if (msg.text().includes('arcgis all ready')) {
              resolve();
            }
          })
        })
      },
      command
    );

  test('exmaples arco-pro run start', async () => {
    await runTest();
  })

  test('exampels arco-pro run preview', async () => {
    await runTest('preview');
  })
});
