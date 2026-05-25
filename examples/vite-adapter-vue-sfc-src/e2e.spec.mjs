import { strict as assert } from 'node:assert';
import { readFile, rm, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { startAndTest } from '../../e2e/index.mjs';

const root = path.resolve(import.meta.dirname);
const componentPath = path.join(root, 'src/components/HelloWorld.vue');
const persistentCachePath = path.join(root, 'node_modules/.farm/vite-vue-cache');

async function assertVueSfcExample(page) {
  await page.waitForSelector('#root');

  assert.match(await page.locator('h1').textContent(), /Farm \+ Vue/);
  assert.match(await page.locator('.card').textContent(), /test HMR/);
  assert.match(await page.locator('.read-the-docs').textContent(), /Farm and Vue logos/);
}

async function waitForStyle(page, selector, property, expected) {
  await page.evaluate(
    ({ selector, property, expected }) =>
      new Promise((resolve, reject) => {
        const deadline = Date.now() + 10000;

        const check = () => {
          const element = document.querySelector(selector);
          const actual = element ? getComputedStyle(element).getPropertyValue(property).trim() : '';
          if (actual === expected) {
            resolve();
          } else if (Date.now() > deadline) {
            reject(new Error(`Expected ${property} to be ${expected}, got ${actual}`));
          } else {
            setTimeout(check, 50);
          }
        };

        check();
      }),
    { selector, property, expected }
  );
}

async function waitForText(page, selector, expected, label) {
  await page.waitForFunction(
    ({ selector, expected }) => document.querySelector(selector)?.textContent?.includes(expected),
    { selector, expected },
    { timeout: 10000 }
  ).catch((error) => {
    throw new Error(`${label}: ${error.message}`);
  });
}

async function withFileEdit(filePath, from, to, run) {
  const original = await readFile(filePath, 'utf8');

  assert.ok(original.includes(from), `Expected ${filePath} to contain ${from}`);
  await writeFile(filePath, original.replace(from, to));

  try {
    await run();
  } finally {
    await writeFile(filePath, original);
  }
}

async function assertCounterState(page) {
  const countButton = page.locator('.card button');
  await countButton.click();
  await waitForText(page, '.card button', 'count is 1', 'counter click did not update');

  return countButton;
}

async function assertTemplateHmr(page, countButton) {
  const button = countButton ?? await assertCounterState(page);

  const textBeforeTemplateHmr = await page.locator('.card').textContent();
  assert.match(textBeforeTemplateHmr, /count is 1/);

  await withFileEdit(
    componentPath,
    '<code>components/HelloWorld.vue</code> to test HMR',
    '<code>components/HelloWorld.vue</code> to test Vite HMR',
    async () => {
      await waitForText(page, '.card', 'test Vite HMR', 'template HMR did not update');
      assert.match(await button.textContent(), /count is 1/);
    }
  );

  await waitForText(page, '.card', 'test HMR', 'template HMR restore did not update');
  assert.match(await button.textContent(), /count is 1/);
}

async function assertStyleHmr(page, countButton) {
  const button = countButton ?? await assertCounterState(page);

  await withFileEdit(componentPath, 'color: #888;', 'color: #1d4ed8;', async () => {
    await waitForStyle(page, '.read-the-docs', 'color', 'rgb(29, 78, 216)');
    assert.match(await button.textContent(), /count is 1/);
  });

  await waitForStyle(page, '.read-the-docs', 'color', 'rgb(136, 136, 136)');
  assert.match(await button.textContent(), /count is 1/);
}

async function assertHmr(page) {
  await assertVueSfcExample(page);

  const countButton = await assertCounterState(page);
  await assertTemplateHmr(page, countButton);
  await assertStyleHmr(page, countButton);
}

async function assertPersistentCacheHmr(page) {
  await assertVueSfcExample(page);

  const countButton = await assertCounterState(page);
  await assertStyleHmr(page, countButton);
  await assertTemplateHmr(page, countButton);
}

export default async function (ctx) {
  const runTest = (command) => startAndTest(root, assertVueSfcExample, command);

  await ctx.test('run start', () => runTest());
  await ctx.test('hmr updates vite vue template and style', () => startAndTest(root, assertHmr));
  await ctx.test('hmr updates vite vue after persistent cache hit', async () => {
    await rm(persistentCachePath, { recursive: true, force: true });
    await startAndTest(root, assertVueSfcExample);
    await startAndTest(root, assertPersistentCacheHmr);
  });
  await ctx.test('run preview', () => runTest('preview'));
}
