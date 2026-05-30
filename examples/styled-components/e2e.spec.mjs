import { basename, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { expect, startAndTest } from '../../e2e/index.mjs';

const name = basename(import.meta.url);
const projectPath = dirname(fileURLToPath(import.meta.url));

async function assertStyledComponentsPluginOutput(page) {
  await page.waitForSelector('[data-testid="styled-components-card"]', { timeout: 10_000 });

  const result = await page.locator('[data-testid="styled-components-card"]').evaluate((node) => {
    const style = getComputedStyle(node);
    return {
      backgroundColor: style.backgroundColor,
      borderRadius: style.borderRadius,
      className: node.className
    };
  });

  expect(result.backgroundColor).toBe('rgb(46, 125, 50)');
  expect(result.borderRadius).toBe('16px');
  expect(result.className).toContain('Main__Card');
}

export default async function (ctx) {
  const runTest = (command) =>
    startAndTest(projectPath, assertStyledComponentsPluginOutput, command);

  await ctx.test(`${name} start`, () => runTest('start'));
  await ctx.test(`${name} preview`, () => runTest('preview'));
}
