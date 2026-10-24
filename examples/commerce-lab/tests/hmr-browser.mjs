import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { performance } from 'node:perf_hooks';
import { chromium } from 'playwright-chromium';

const baseURL = process.env.COMMERCE_URL ?? 'http://localhost:4320';
const source = fileURLToPath(new URL('../apps/admin/pages/Components.tsx', import.meta.url));
const original = await fs.readFile(source, 'utf8');
const marker = 'Commerce HMR leaf verification';
assert.ok(original.includes('Built to work together'), 'Leaf edit anchor missing');
const browser = await chromium.launch({ headless: true });
let changed = false;
const diagnostics = { pageErrors: [], consoleErrors: [] };
let page;
try {
  page = await browser.newPage();
  const errors = [];
  page.on('pageerror', error => { errors.push(error.message); diagnostics.pageErrors.push(error.message); });
  page.on('console', message => { if (message.type() === 'error') diagnostics.consoleErrors.push(message.text()); });
  await page.goto(`${baseURL}/#/components`);
  await page.getByRole('button', { name: 'Interactive SVG' }).click();
  await page.getByText('Icon clicked 1 time', { exact: true }).waitFor();
  await page.evaluate(() => { window.__commerceHmrDocument = 'original-document'; });
  const started = performance.now();
  changed = true;
  await fs.writeFile(source, original.replace('Built to work together', marker));
  await page.getByRole('heading', { name: marker, exact: true }).waitFor({ timeout: 30000 });
  const durationMs = performance.now() - started;
  const sameDocument = await page.evaluate(() => window.__commerceHmrDocument === 'original-document');
  const statePreserved = await page.getByText('Icon clicked 1 time', { exact: true }).count() === 1;
  console.log(JSON.stringify({ baseURL, scenario: 'hmr-leaf', durationMs, sameDocument, statePreserved, errors }));
  assert.deepEqual(errors, []);
  assert.equal(sameDocument, true, 'Leaf HMR must not reload the document');
  assert.equal(statePreserved, true, 'Leaf HMR must preserve component state');
  await fs.writeFile(source, original);
  await page.getByRole('heading', { name: 'Built to work together', exact: true }).waitFor({ timeout: 30000 });
  changed = false;
} catch (error) {
  console.log(JSON.stringify({ success: false, baseURL, scenario: 'hmr-leaf', error: error.message,
    ...diagnostics, pageText: page ? await page.locator('body').innerText().catch(() => '') : '' }));
  throw error;
} finally {
  if (changed) await fs.writeFile(source, original);
  await browser.close();
}
