import { expect, test } from 'vitest';
import { existsSync, readdirSync, statSync } from 'fs';
import { concurrentMap } from './utils';
import { join } from 'path';
import { startProjectAndTest } from './vitestSetup';

const excludeExamples: string[] = [
  // todo add e2e test for ssr
  'react-ssr',
  'vue-ssr',
  'solid-ssr'
];

async function startTest() {
  const examples = readdirSync('./examples');
  await Promise.all(
    concurrentMap(examples, 50, async (example: string) => {
      const examplePath = join('./examples', example);
      const hasE2eTestFile = existsSync(join(examplePath, 'e2e.spec.ts'));
      // TODO: add e2e.spec.ts for library examples
      const hasIndexHtml = existsSync(join(examplePath, 'index.html'));

      if (
        hasE2eTestFile ||
        excludeExamples.includes(example) ||
        !hasIndexHtml
      ) {
        return;
      }

      if (statSync(examplePath).isDirectory()) {
        return startProjectAndTest(examplePath, async (page) => {
          // id root should be in the page
          await page.waitForSelector('#root > *', { timeout: 10000 });
          const child = await page.$('#root > *');
          expect(child).toBeTruthy();
        });
      }
      return;
    })
  );
}

test('Default E2E Tests', async () => {
  await startTest();
});
