import fs from 'node:fs/promises';
import { chromium } from 'playwright-chromium';

const file = new URL('./remote-a/lazy.js', import.meta.url);
const original = await fs.readFile(file, 'utf8');
const browser = await chromium.launch({ headless: true });
const page = await browser.newPage();
const errors = [];
page.on('pageerror', error => errors.push(error.message));
async function reloadUntil(expected) {
  const deadline = Date.now() + 15000;
  let lastError;
  do {
    try {
      await page.goto('http://127.0.0.1:4100');
      await page.waitForFunction(() => window.probe?.ready, {}, {timeout:2000});
      await page.locator('#button-a').click();
      await page.waitForFunction(text => document.querySelector('#button-a')?.textContent === text, expected, {timeout:700});
      return;
    } catch (error) { lastError = error; }
  } while (Date.now() < deadline);
  throw lastError;
}
const evidence = {passed:false, errors};
try {
  await reloadUntil('lazy-a:1');
  await fs.writeFile(file, original.replace('lazy-a', 'lazy-a-updated'));
  await reloadUntil('lazy-a-updated:1');
  evidence.updatedAfterFullPageReload = true;
  await fs.writeFile(file, original);
  await reloadUntil('lazy-a:1');
  evidence.restoredAfterFullPageReload = true;
  if (errors.length) throw new Error(JSON.stringify(errors));
  evidence.passed = true;
} catch (error) {
  evidence.failure = error.stack;
  process.exitCode = 1;
} finally {
  await fs.writeFile(file, original);
  await browser.close();
  await fs.writeFile(new URL('./evidence-refresh.json', import.meta.url), JSON.stringify(evidence,null,2));
  console.log(JSON.stringify(evidence,null,2));
}
