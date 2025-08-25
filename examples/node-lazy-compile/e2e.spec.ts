import { test, describe } from 'vitest';
import { watchProjectAndTest } from '../../e2e/vitestSetup.js';
import { basename, dirname } from 'path';
import { fileURLToPath } from 'url';
import { exec } from 'child_process';

const name = basename(import.meta.url);
const projectPath = dirname(fileURLToPath(import.meta.url));

describe(`e2e tests - ${name}`, async () => {
  const runTest = (command?: 'watch' | 'preview') =>
    watchProjectAndTest(
      projectPath,
      async (log, done) => {
        if (command === 'preview') {
          console.log(log);
          if (log.includes('script start') && log.includes('111aaa')) {
            done();
          }
        } else {
          if (log.includes('Build completed in')) {
            const output = await new Promise<string>((resolve, reject) => {
              exec('npm run preview', {
                cwd: projectPath
              }, (error, stdout) => {
                if (error) {
                  reject(error);
                }
                resolve(stdout);
              })
            });

            if (output.includes('script start') && output.includes('111aaa')) {
              done();
            }
          }
        }
      },
      command
    );

  // preview build
  test('preview', async () => {
    await runTest('preview');
  });

  test('watch', async () => {
    await runTest('watch');
  });
});
