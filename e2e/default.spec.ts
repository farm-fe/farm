import { expect, test } from 'vitest';
import { existsSync, readdirSync, statSync } from 'fs';
import { join } from 'path';
import { startProjectAndTest } from './vitestSetup.js';
import { logger } from './utils.js';
// import { ssrExamples } from './test-utils.js';

const excludeExamples: string[] = [];

test('Default E2E Tests', async () => {
  const examples = readdirSync('./examples');
  // const examples = ['react-ssr', 'solid-ssr', 'vue-ssr'];
  logger(`Running E2E tests for ${examples.length} examples`);

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
        'excludeExamples',
        excludeExamples,
        'hasIndexHtml',
        hasIndexHtml
      );
      continue;
    }

    console.log(`Testing ${example}`);

    if (statSync(examplePath).isDirectory()) {
      const runTest = (command?: 'start' | 'preview') =>
        startProjectAndTest(
          examplePath,
          async (page) => {
            // id root should be in the page
            await page.waitForSelector('#root > *', { timeout: 10000 });
            const child = await page.$('#root > *');
            expect(child).toBeTruthy();
          },
          command
        );

      await runTest();
      await runTest('preview');
    }
  }
});
