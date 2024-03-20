import { expect, test } from 'vitest';
import { startProjectAndTest } from '../../e2e/vitestSetup';
import { basename, dirname } from 'path';
import { fileURLToPath } from 'url';

const name = basename(import.meta.url);
const projectPath = dirname(fileURLToPath(import.meta.url));

test(`e2e tests - ${name}`, async () => {
  await startProjectAndTest(projectPath, async (page) => {
    const app = await page.$('#app');
    expect(await app?.innerHTML()).toBe('app');
  });
});
