import { test, describe, expect } from 'vitest';
import { startProjectAndTest, watchProjectAndTest } from '../../e2e/vitestSetup';
import { basename, dirname } from 'path';
import { fileURLToPath } from 'url';
import { execSync } from 'child_process';

const name = basename(import.meta.url);
const projectPath = dirname(fileURLToPath(import.meta.url));

describe(`e2e tests - ${name}`, async () => {
  const runTest = (command: 'start' | 'preview' = 'start') =>
    startProjectAndTest(
      projectPath,
      async (page) => {
        const root = await page.waitForSelector('#root');
        const img = await root.waitForSelector('img');
        expect(await img.getAttribute('src')).contains('logo');
      },
      command
    );

  test('exmaples arco-pro run start', async () => {
    await watchProjectAndTest(projectPath, async (log, done) => {
      console.log(log);
      if (log.includes('Build completed in')) {
        await runTest();
        done();
      }
    }, 'watch');
  },)

  test('exampels arco-pro run preview', async () => {
    execSync('npm run build', {
      cwd: projectPath,
      stdio: 'inherit'
    });
    await runTest('preview');
  })
});
