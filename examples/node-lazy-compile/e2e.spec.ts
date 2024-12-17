import { test } from 'vitest';
import { watchProjectAndTest } from '../../e2e/vitestSetup.js';
import { basename, dirname } from 'path';
import { fileURLToPath } from 'url';

const name = basename(import.meta.url);
const projectPath = dirname(fileURLToPath(import.meta.url));

test(`e2e tests - ${name}`, async () => {
  const runTest = (command?: 'watch' | 'preview') =>
    watchProjectAndTest(
      projectPath,
      async (log, done) => {
        if (command === 'preview') {
          if (log.includes('script start') && log.includes('111aaa')) {
            done();
          }
        } else {
          if (log.includes('Build completed in')) {
            done();
          }
        }
      },
      command
    );

  // preview build
  await runTest('preview');
  await runTest('watch');
  await runTest('preview');
});
