import { basename, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { editFile, expect, startAndTest } from '../../e2e/index.mjs';

const name = basename(import.meta.url);
const projectPath = dirname(fileURLToPath(import.meta.url));

async function assertPage(page) {
  await page.waitForSelector('#root > *', { timeout: 10_000 });

  const headingText = await page.textContent('h1');
  expect(headingText).toContain('TailwindCSS Rust Plugin');

  for (const id of [
    'alert-section',
    'button-section',
    'card-section',
    'swatch-section'
  ]) {
    const el = await page.$(`[data-testid="${id}"]`);
    expect(el).toBeTruthy();
  }

  for (const btn of ['btn-primary', 'btn-secondary', 'btn-danger']) {
    const el = await page.$(`[data-testid="${btn}"]`);
    expect(el).toBeTruthy();
  }

  const primaryBg = await page.evaluate(() => {
    const el = document.querySelector('[data-testid="btn-primary"]');
    return el ? window.getComputedStyle(el).backgroundColor : '';
  });
  expect(primaryBg).toBe('rgb(18, 52, 86)');

  const alertBg = await page.evaluate(() => {
    const el = document.querySelector('[data-testid="applied-alert"]');
    return el ? window.getComputedStyle(el).backgroundColor : '';
  });
  expect(alertBg).not.toBe('rgba(0, 0, 0, 0)');
  expect(alertBg).not.toBe('');

  const swatchCount = await page.evaluate(
    () => document.querySelectorAll('[data-testid="swatch-grid"] [data-color]').length
  );
  expect(swatchCount).toBe(8);
}

export default async function (ctx) {
  const runTest = (command = 'start') =>
    startAndTest(projectPath, async (page) => {
      await assertPage(page);
    }, command);

  await ctx.test(`example ${name} run start`, async () => {
    await runTest();
  });

  await ctx.test(`example ${name} run preview`, async () => {
    await runTest('preview');
  });

  await ctx.test(`example ${name} updates Tailwind utilities during HMR`, async () => {
    await startAndTest(projectPath, async (page) => {
      await assertPage(page);

      const restoreButton = await editFile(
        join(projectPath, 'src/components/Button.tsx'),
        'bg-brand hover:bg-brand-dark text-white',
        'bg-accent hover:bg-brand-dark text-white'
      );

      try {
        await page.waitForFunction(() => {
          const el = document.querySelector('[data-testid="btn-primary"]');
          return el && window.getComputedStyle(el).backgroundColor === 'rgb(171, 205, 239)';
        });
      } finally {
        await restoreButton?.();
      }
    });
  });
}
