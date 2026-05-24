import { test, expect, describe } from 'vitest';
import { startProjectAndTest } from '../../e2e/vitestSetup';
import { basename, dirname } from 'path';
import { fileURLToPath } from 'url';

const name = basename(import.meta.url);
const projectPath = dirname(fileURLToPath(import.meta.url));

describe(`e2e tests - ${name}`, async () => {
  const runTest = (command?: 'start' | 'preview') =>
    startProjectAndTest(
      projectPath,
      async (page) => {
        if (command === 'start') {
          await page.waitForTimeout(3000);
        } else {
          await page.waitForTimeout(1000);
        }

        await page.waitForSelector('#root > *', { timeout: 10000 });
        const child = await page.$('#root > *');
        expect(child).toBeTruthy();

        // ── 1. Heading exists ─────────────────────────────────────────
        const heading = await page.$('h1');
        expect(heading).toBeTruthy();
        const headingText = await heading?.textContent();
        expect(headingText).toContain('TailwindCSS Rust Plugin');

        // ── 2. All sections are rendered ──────────────────────────────
        for (const id of [
          'alert-section',
          'button-section',
          'card-section',
          'swatch-section'
        ]) {
          const el = await page.$(`[data-testid="${id}"]`);
          expect(el, `section ${id} should render`).toBeTruthy();
        }

        // ── 3. Variant buttons exist ──────────────────────────────────
        for (const btn of ['btn-primary', 'btn-secondary', 'btn-danger']) {
          const el = await page.$(`[data-testid="${btn}"]`);
          expect(el, `button ${btn} should render`).toBeTruthy();
        }

        // ── 4. Tailwind utility classes resolve to real CSS at runtime ─
        // `bg-blue-500` is a representative functional color utility.
        // If the rust compiler did not emit the rule, getComputedStyle
        // would return the default `rgba(0, 0, 0, 0)`.
        const primaryBg = await page.evaluate(() => {
          const el = document.querySelector('[data-testid="btn-primary"]');
          return el
            ? window.getComputedStyle(el as Element).backgroundColor
            : '';
        });
        expect(primaryBg, 'btn-primary should have a non-transparent bg').not.toBe(
          'rgba(0, 0, 0, 0)'
        );
        expect(primaryBg, 'btn-primary should have a non-transparent bg').not.toBe(
          ''
        );

        // ── 5. @apply rule was processed by the rust plugin ───────────
        // `.applied-alert` is composed of @apply utilities in index.css.
        const alertBg = await page.evaluate(() => {
          const el = document.querySelector('[data-testid="applied-alert"]');
          return el
            ? window.getComputedStyle(el as Element).backgroundColor
            : '';
        });
        expect(alertBg, 'applied-alert background should be set').not.toBe(
          'rgba(0, 0, 0, 0)'
        );
        expect(alertBg, 'applied-alert background should be set').not.toBe('');

        // ── 6. Swatch grid contains every swatch ──────────────────────
        const swatchCount = await page.evaluate(
          () =>
            document.querySelectorAll(
              '[data-testid="swatch-grid"] [data-color]'
            ).length
        );
        expect(swatchCount).toBe(8);
      },
      command
    );

  test(`example ${name} run start`, async () => {
    await runTest();
  });

  test(`example ${name} run preview`, async () => {
    await runTest('preview');
  });
});
