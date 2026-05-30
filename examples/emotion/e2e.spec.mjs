import { basename, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { expect, startAndTest } from '../../e2e/index.mjs';

const name = basename(import.meta.url);
const projectPath = dirname(fileURLToPath(import.meta.url));

async function assertEmotionPluginOutput(page) {
  await page.waitForSelector('[data-testid="emotion-card"]', { timeout: 10_000 });

  const result = await page.locator('[data-testid="emotion-card"]').evaluate((node) => {
    const style = getComputedStyle(node);
    return {
      backgroundColor: style.backgroundColor,
      borderRadius: style.borderRadius,
      className: node.className
    };
  });

  expect(result.backgroundColor).toBe('rgb(255, 105, 180)');
  expect(result.borderRadius).toBe('4px');
  expect(result.className).toContain('farm-emotion');
}

export default async function (ctx) {
  const runTest = (command) =>
    startAndTest(projectPath, assertEmotionPluginOutput, command);

  await ctx.test(`${name} start`, () => runTest('start'));
  await ctx.test(`${name} preview`, () => runTest('preview'));
}
