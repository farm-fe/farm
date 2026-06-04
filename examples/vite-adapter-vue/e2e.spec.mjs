import { dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { expect, startAndTest } from '../../e2e/index.mjs';

const projectPath = dirname(fileURLToPath(import.meta.url));

function collectRequestIssues(page) {
  const requestIssues = [];

  page.on('requestfailed', (req) => {
    requestIssues.push(`${req.url()} ${req.failure()?.errorText || ''}`);
  });

  return requestIssues;
}

async function assertDeferImport(page, requestIssues) {
  await page.waitForSelector('#root > *', { timeout: 10_000 });
  await page.waitForSelector('#defer-message', { timeout: 10_000 });
  expect(await page.textContent('#defer-message')).toBe('Deferred import evaluation works');
  expect(requestIssues).toEqual([]);
}

export default async function (ctx) {
  const runTest = (command) =>
    startAndTest(
      projectPath,
      async (page) => {
        await assertDeferImport(page, collectRequestIssues(page));
      },
      command
    );

  await ctx.test('deferred import evaluation in start', () => runTest());
  await ctx.test('deferred import evaluation in preview', () => runTest('preview'));
}
