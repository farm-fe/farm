import { startAndTest, expect } from '../../e2e/index.mjs';
import { readFile, rm, writeFile } from 'node:fs/promises';
import { dirname } from 'path';
import { fileURLToPath } from 'url';
import { join } from 'node:path';

const projectPath = dirname(fileURLToPath(import.meta.url));
const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const homeViewPath = join(projectPath, 'src/views/HomeView.vue');
const counterCardPath = join(projectPath, 'src/components/CounterCard.vue');
const welcomePath = join(projectPath, 'src/components/Welcome.tsx');
const persistentCachePath = join(projectPath, 'node_modules/.farm/vue-cache');
const hmrTimeout = Number(process.env.FARM_E2E_HMR_TIMEOUT ?? 30_000);
const hmrPollInterval = 100;

function collectRequestIssues(page) {
  const requestIssues = [];

  page.on('requestfailed', (req) => {
    requestIssues.push(`${req.url()} ${req.failure()?.errorText || ''}`);
  });

  return requestIssues;
}

async function assertVueExample(page, requestIssues) {
  await page.waitForSelector('#root > *', { timeout: 10_000 });
  expect(await page.textContent('h1')).toBe('Farm + Vue');
  expect(await page.textContent('.intro')).toContain('@farmfe/plugin-vue');
  expect(await page.textContent('.card strong')).toContain('Pinia count: 0');

  const homeStyles = await page.$eval('section', (section) => {
    const intro = section.querySelector('.intro');
    const card = section.querySelector('.card');
    const button = section.querySelector('.card button');
    return {
      introScopeAttrs: intro
        ? Array.from(intro.attributes).map((attr) => attr.name).filter((name) => name.startsWith('data-v-'))
        : [],
      cardScopeAttrs: card
        ? Array.from(card.attributes).map((attr) => attr.name).filter((name) => name.startsWith('data-v-'))
        : [],
      introColor: intro ? getComputedStyle(intro).color : '',
      cardDisplay: card ? getComputedStyle(card).display : '',
      cardBackground: card ? getComputedStyle(card).backgroundColor : '',
      buttonBackground: button ? getComputedStyle(button).backgroundColor : '',
      buttonColor: button ? getComputedStyle(button).color : ''
    };
  });

  expect(homeStyles.introScopeAttrs.length).toBeGreaterThan(0);
  expect(homeStyles.cardScopeAttrs.length).toBeGreaterThan(0);
  expect(homeStyles.introColor).toBe('rgb(45, 106, 79)');
  expect(homeStyles.cardDisplay).toBe('flex');
  expect(homeStyles.cardBackground).toBe('rgba(64, 145, 108, 0.12)');
  expect(homeStyles.buttonBackground).toBe('rgb(64, 145, 108)');
  expect(homeStyles.buttonColor).toBe('rgb(255, 255, 255)');

  await page.locator('.card button').click();
  expect(await page.textContent('.card strong')).toContain('Pinia count: 1');
  // JSX component verification
  const jsxBadge = await page.$eval('.jsx-badge', (el) => el.textContent);
  expect(jsxBadge).toContain('@farmfe/plugin-vue-jsx');

  const jsxCount = await page.$eval('.jsx-card strong', (el) => el.textContent);
  expect(jsxCount).toContain('JSX count: 0');

  await page.locator('.jsx-card button').first().click();
  await delay(200);
  const updatedCount = await page.$eval('.jsx-card strong', (el) => el.textContent);
  expect(updatedCount).toContain('JSX count: 1');

  await page.locator('.jsx-card button').nth(1).click();
  await delay(200);
  const revealText = await page.$eval('.jsx-reveal', (el) => el.textContent);
  expect(revealText).toContain('v-show directive works!');

  await page.locator('a[href="#/about"]').click();
  await page.waitForSelector('.about', { timeout: 10_000 });
  expect(await page.textContent('.about')).toContain('router navigation');

  await delay(500);
  expect(requestIssues).toEqual([]);
}

async function withFileEdits(edits, run) {
  const originals = [];

  try {
    for (const edit of edits) {
      const content = await readFile(edit.file, 'utf8');
      originals.push({ file: edit.file, content });
      const next = content.replace(edit.from, edit.to);
      if (next === content) {
        throw new Error(`HMR fixture edit did not match in ${edit.file}`);
      }
      await writeFile(edit.file, next);
    }

    await run();
  } finally {
    await Promise.all(originals.map(({ file, content }) => writeFile(file, content)));
  }
}

async function waitForStyle(page, selector, property, expected) {
  const deadline = Date.now() + hmrTimeout;
  let actual = '';

  while (Date.now() < deadline) {
    actual = await page
      .$eval(selector, (element, property) => getComputedStyle(element)[property], property)
      .catch(() => '');
    if (actual === expected) return;
    await delay(hmrPollInterval);
  }

  throw new Error(`Expected ${selector} ${property} to be ${expected}, got ${actual}`);
}

async function waitForText(page, selector, expected) {
  const deadline = Date.now() + hmrTimeout;
  let actual = '';

  while (Date.now() < deadline) {
    actual = await page.textContent(selector).catch(() => '');
    if (actual?.includes(expected)) return;
    await delay(hmrPollInterval);
  }

  throw new Error(`Expected ${selector} to contain ${expected}, got ${actual}`);
}

async function assertHmr(page) {
  await page.locator('a[href="#/"]').click();
  await page.waitForSelector('.card button', { timeout: 10_000 });
  await waitForText(page, '.intro', 'This example uses');
  await delay(500);
  if ((await page.textContent('.card strong')).includes('Pinia count: 0')) {
    await page.locator('.card button').click();
  }
  expect(await page.textContent('.card strong')).toContain('Pinia count: 1');

  await withFileEdits(
    [
      {
        file: homeViewPath,
        from: 'This example uses the native Rust',
        to: 'HMR template update uses the native Rust'
      }
    ],
    async () => {
      await waitForText(page, '.intro', 'HMR template update');
      expect(await page.textContent('.card strong')).toContain('Pinia count: 1');
    }
  );
  await waitForText(page, '.intro', 'This example uses');

  await withFileEdits(
    [
      {
        file: homeViewPath,
        from: 'color: #2d6a4f;',
        to: 'color: #1d4ed8;'
      }
    ],
    async () => {
      await waitForStyle(page, '.intro', 'color', 'rgb(29, 78, 216)');
      expect(await page.textContent('.card strong')).toContain('Pinia count: 1');
    }
  );
  await waitForStyle(page, '.intro', 'color', 'rgb(45, 106, 79)');

  // JSX HMR test
  await withFileEdits(
    [
      {
        file: welcomePath,
        from: 'Rendered by',
        to: 'HMR-updated JSX powered by'
      }
    ],
    async () => {
      await waitForText(page, '.jsx-badge', 'HMR-updated JSX');
      expect(await page.textContent('.jsx-card strong')).toContain('JSX count: 0');
    }
  );
  await waitForText(page, '.jsx-badge', 'Rendered by');

  await withFileEdits(
    [
      {
        file: counterCardPath,
        from: '$accent: #40916c;',
        to: '$accent: #1d4ed8;'
      }
    ],
    async () => {
      await waitForStyle(page, '.card', 'backgroundColor', 'rgba(29, 78, 216, 0.12)');
      await waitForStyle(page, '.card button', 'backgroundColor', 'rgb(29, 78, 216)');
      expect(await page.textContent('.card strong')).toContain('Pinia count: 1');
    }
  );
}

export default async function (ctx) {
  const runTest = (command) =>
    startAndTest(
      projectPath,
      async (page) => {
        const requestIssues = collectRequestIssues(page);
        await assertVueExample(page, requestIssues);
      },
      command
    );

  await ctx.test('run start', () => runTest());
  await ctx.test('hmr updates template and styles', () =>
    startAndTest(projectPath, async (page) => {
      await assertVueExample(page, collectRequestIssues(page));
      await assertHmr(page);
    })
  );
  await ctx.test('hmr updates after persistent cache hit', async () => {
    await rm(persistentCachePath, { recursive: true, force: true });
    await startAndTest(projectPath, async (page) => {
      await assertVueExample(page, collectRequestIssues(page));
    });
    await startAndTest(projectPath, async (page) => {
      await assertVueExample(page, collectRequestIssues(page));
      await assertHmr(page);
    });
  });
  await ctx.test('run preview', () => runTest('preview'));
}
