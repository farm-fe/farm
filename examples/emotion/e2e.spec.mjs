import { basename, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { expect, startAndTest } from '../../e2e/index.mjs';

const name = basename(import.meta.url);
const projectPath = dirname(fileURLToPath(import.meta.url));

async function assertEmotionPluginOutput(page) {
  await page.waitForSelector('[data-testid="emotion-card"]', { timeout: 10_000 });
  await page.waitForSelector('[data-testid="styled-components-card"]', { timeout: 10_000 });
  await page.waitForSelector('[data-testid="styled-jsx-card"]', { timeout: 10_000 });

  const emotionResult = await page.locator('[data-testid="emotion-card"]').evaluate((node) => {
    const style = getComputedStyle(node);
    return {
      backgroundColor: style.backgroundColor,
      borderRadius: style.borderRadius,
      className: node.className
    };
  });

  expect(emotionResult.backgroundColor).toBe('rgb(255, 105, 180)');
  expect(emotionResult.borderRadius).toBe('4px');
  expect(emotionResult.className).toContain('css-');

  const styledComponentsResult = await page.locator('[data-testid="styled-components-card"]').evaluate((node) => {
    const style = getComputedStyle(node);
    return {
      backgroundColor: style.backgroundColor,
      borderRadius: style.borderRadius,
      className: node.className
    };
  });

  expect(styledComponentsResult.backgroundColor).toBe('rgb(46, 125, 50)');
  expect(styledComponentsResult.borderRadius).toBe('16px');

  const styledJsxResult = await page.locator('[data-testid="styled-jsx-card"]').evaluate((node) => {
    const style = getComputedStyle(node);
    return {
      backgroundColor: style.backgroundColor,
      borderRadius: style.borderRadius,
      className: node.className
    };
  });

  expect(styledJsxResult.backgroundColor).toBe('rgb(25, 118, 210)');
  expect(styledJsxResult.borderRadius).toBe('12px');
  expect(styledJsxResult.className).toContain('jsx-');
}

export default async function (ctx) {
  const runTest = (command) =>
    startAndTest(projectPath, assertEmotionPluginOutput, command);

  await ctx.test(`${name} start`, () => runTest('start'));
  await ctx.test(`${name} preview`, () => runTest('preview'));
}
