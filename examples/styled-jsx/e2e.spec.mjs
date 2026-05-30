import { basename, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { expect, startAndTest } from '../../e2e/index.mjs';

const name = basename(import.meta.url);
const projectPath = dirname(fileURLToPath(import.meta.url));

async function assertStyledJsxPluginOutput(page) {
  await page.waitForSelector('[data-testid="styled-jsx-card"]', { timeout: 10_000 });

  const result = await page.locator('[data-testid="styled-jsx-card"]').evaluate((node) => {
    const style = getComputedStyle(node);
    return {
      backgroundColor: style.backgroundColor,
      borderRadius: style.borderRadius,
      className: node.className
    };
  });

  expect(result.backgroundColor).toBe('rgb(25, 118, 210)');
  expect(result.borderRadius).toBe('12px');
  expect(result.className).toContain('jsx-');
}

export default async function (ctx) {
  const runTest = (command) =>
    startAndTest(projectPath, assertStyledJsxPluginOutput, command);

  await ctx.test(`${name} start`, () => runTest('start'));
  await ctx.test(`${name} preview`, () => runTest('preview'));
}
