import { startAndTest, watchAndTest, expect } from '../../e2e/index.mjs';
import { dirname } from 'path';
import { fileURLToPath } from 'url';
import { execSync } from 'child_process';

const projectPath = dirname(fileURLToPath(import.meta.url));

export default async function (ctx) {
  const runStart = () =>
    startAndTest(
      projectPath,
      async (page) => {
        const root = await page.waitForSelector('#root');
        const img = await root.waitForSelector('img');
        expect(await img.getAttribute('src')).contains('logo');
      },
      'start'
    );

  await ctx.test('run start (via watch)', async () => {
    await watchAndTest(
      projectPath,
      async (log, done) => {
        if (log.includes('Build completed in')) {
          await runStart();
          done();
        }
      },
      'watch'
    );
  });

  await ctx.test('run preview', async () => {
    execSync('npm run build', { cwd: projectPath, stdio: 'inherit' });
    await startAndTest(
      projectPath,
      async (page) => {
        const root = await page.waitForSelector('#root');
        const img = await root.waitForSelector('img');
        expect(await img.getAttribute('src')).contains('logo');
      },
      'preview'
    );
  });
}
