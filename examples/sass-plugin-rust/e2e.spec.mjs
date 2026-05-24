import { editFile, expect, startAndTest } from '../../e2e/index.mjs';
import path, { dirname } from 'path';
import { fileURLToPath } from 'url';

const projectPath = dirname(fileURLToPath(import.meta.url));

const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function collectPageIssues(page) {
  const consoleIssues = [];
  const requestIssues = [];

  page.on('console', (msg) => {
    if (msg.type() === 'error') {
      consoleIssues.push(`${msg.type()}: ${msg.text()}`);
    }
  });

  page.on('pageerror', (error) => {
    consoleIssues.push(`pageerror: ${error.message}`);
  });

  page.on('requestfailed', (req) => {
    requestIssues.push(`${req.url()} ${req.failure()?.errorText || ''}`);
  });

  return { consoleIssues, requestIssues };
}

async function assertSassOutput(page) {
  await page.waitForSelector('[data-testid="sass-dep"]', { timeout: 10_000 });

  const depText = await page.textContent('[data-testid="sass-dep"]');
  expect(depText).toContain('sass');

  const depStyles = await page.$eval('[data-testid="sass-dep"]', (el) => {
    const style = getComputedStyle(el);
    return {
      backgroundImage: style.backgroundImage,
      backgroundRepeat: style.backgroundRepeat,
      backgroundSize: style.backgroundSize,
      color: style.color,
      height: style.height,
      width: style.width
    };
  });

  expect(depStyles.color).toBe('rgb(0, 0, 255)');
  expect(depStyles.width).toBe('200px');
  expect(depStyles.height).toBe('50px');
  expect(depStyles.backgroundRepeat).toBe('no-repeat');
  expect(depStyles.backgroundSize).toBe('contain');
  expect(depStyles.backgroundImage).toContain('logo');

  await page.hover('[data-testid="sass-description"]');
  await page.waitForFunction(() => {
    const el = document.querySelector('[data-testid="sass-description"]');
    return el && getComputedStyle(el).color === 'rgb(241, 2, 21)';
  });
}

async function assertNoPageIssues(pageIssues) {
  await delay(500);
  expect(pageIssues.consoleIssues).toEqual([]);
  expect(pageIssues.requestIssues).toEqual([]);
}

export default async function (ctx) {
  const runTest = (command) =>
    startAndTest(
      projectPath,
      async (page) => {
        const pageIssues = await collectPageIssues(page);

        await assertSassOutput(page);
        await assertNoPageIssues(pageIssues);
      },
      command
    );

  await ctx.test('run start', () => runTest());
  await ctx.test('run preview', () => runTest('preview'));

  await ctx.test('run start (Sass HMR)', () =>
    startAndTest(projectPath, async (page) => {
      const pageIssues = await collectPageIssues(page);
      const variablePath = path.join(projectPath, 'src/style/variables.scss');

      await assertSassOutput(page);

      const recover = await editFile(variablePath, '$test:blue;', '$test:rgb(255, 0, 0);');
      try {
        await page.waitForFunction(() => {
          const el = document.querySelector('[data-testid="sass-dep"]');
          return el && getComputedStyle(el).color === 'rgb(255, 0, 0)';
        });
      } finally {
        await recover?.();
      }

      await assertNoPageIssues(pageIssues);
    })
  );
}
