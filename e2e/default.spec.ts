import { describe } from 'node:test';
import { existsSync, readdirSync, statSync } from 'fs';
import { join } from 'path';
import { expect, test } from 'vitest';
import { logger } from './utils.js';
import { startProjectAndTest } from './vitestSetup.js';

// import { ssrExamples } from './test-utils.js';

const excludeExamples: string[] = ['issues1433', 'nestjs'];

describe('Default E2E Tests', async () => {
  const examples = readdirSync('./examples');
  // const examples = ['react-ssr', 'solid-ssr', 'vue-ssr'];
  // const examples = ['module-concatenation', 'tailwind-next'];
  logger(`Running E2E tests for ${examples.length} examples`);

  console.log('exclude examples', excludeExamples);

  for (const example of examples) {
    const examplePath = join('./examples', example);
    const hasE2eTestFile = existsSync(join(examplePath, 'e2e.spec.ts'));
    // TODO: add e2e.spec.ts for library examples
    const hasIndexHtml = existsSync(join(examplePath, 'index.html'));

    if (hasE2eTestFile || excludeExamples.includes(example) || !hasIndexHtml) {
      console.log(
        'skip',
        example,
        'hasE2eTestFile',
        hasE2eTestFile,
        'hasIndexHtml',
        hasIndexHtml
      );
      continue;
    }

    if (!statSync(examplePath).isDirectory()) {
      return;
    }

    const runTest = (command?: 'start' | 'preview') =>
      startProjectAndTest(
        examplePath,
        async (page) => {
          if (command === 'start') {
            // wait 3s for the dynamic import to be loaded
            await page.waitForTimeout(3000);
          } else {
            // wait 1s for the dynamic import to be loaded
            await page.waitForTimeout(1000);
          }

          // id root should be in the page
          await page.waitForSelector('#root > *', { timeout: 10000 });
          const child = await page.$('#root > *');
          expect(child).toBeTruthy();
        },
        command
      );

    test(`test example ${example} start`, async () => {
      await runTest();
    });

    test(`test example ${example} preview`, async () => {
      await runTest('preview');
    });
  }
});
