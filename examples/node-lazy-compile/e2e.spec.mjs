import { watchAndTest } from '../../e2e/index.mjs';
import { dirname } from 'path';
import { fileURLToPath } from 'url';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);
const projectPath = dirname(fileURLToPath(import.meta.url));

export default async function (ctx) {
  await ctx.test('run preview', async () => {
    await watchAndTest(
      projectPath,
      async (log, done) => {
        if (log.includes('script start') && log.includes('111aaa')) {
          done();
        }
      },
      'preview'
    );
  });

  await ctx.test('run watch + preview', async () => {
    await watchAndTest(
      projectPath,
      async (log, done) => {
        if (log.includes('Build completed in')) {
          const { stdout } = await execAsync('npm run preview', { cwd: projectPath });
          if (stdout.includes('script start') && stdout.includes('111aaa')) {
            done();
          }
        }
      },
      'watch'
    );
  });
}
