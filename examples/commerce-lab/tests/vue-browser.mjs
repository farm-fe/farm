import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { chromium } from 'playwright-chromium';

const browser = await chromium.launch({ headless: true });
const page = await browser.newPage();
const errors = [];
page.on('pageerror', error => errors.push(error.message));
const variant = process.env.COMMERCE_VUE_VARIANT ?? 'sfc';
assert.ok(['sfc', 'jsx'].includes(variant));
const result = { kind: 'vue-business-correctness', variant, group: process.env.COMMERCE_GROUP ?? 'js', success: false, pageErrors: errors };
try {
  await page.goto(process.env.COMMERCE_URL ?? 'http://localhost:4330');
  await page.getByRole('heading', { name: 'Merchant inventory' }).waitFor({ timeout: 5000 });
  assert.equal(await page.getByTestId('inventory-row').count(), 60);
  await page.getByLabel('Search inventory').fill('NORTH-1000');
  assert.equal(await page.getByTestId('inventory-row').count(), 1);
  const row = page.getByTestId('inventory-row');
  const stock = Number(await row.getByTestId('stock').innerText());
  await row.getByRole('button', { name: 'Receive 25 units' }).click();
  await row.getByTestId('stock').filter({ hasText: new RegExp(`^${stock + 25}$`) }).waitFor({ timeout: 5000 });
  assert.equal(Number(await row.getByTestId('stock').innerText()), stock + 25);
  await page.getByLabel('Role').selectOption('viewer');
  assert.ok(await row.getByRole('button', { name: 'Receive 25 units' }).isDisabled());
  await page.getByLabel('Merchant').selectOption('south');
  assert.equal(await page.getByTestId('inventory-row').count(), 0);
  await page.getByLabel('Search inventory').fill('');
  assert.equal(await page.getByTestId('inventory-row').count(), 60);
  assert.match(await page.getByTestId('inventory-row').first().innerText(), /SOUTH-1000/);
  await page.getByLabel('Low stock only').check();
  const stocks = await page.getByTestId('stock').allTextContents();
  assert.ok(stocks.length > 0 && stocks.every(value => Number(value) < 25));
  result.scopedStyle = await page.getByRole('heading', { name: 'Merchant inventory' }).evaluate(node => getComputedStyle(node).color);
  assert.equal(result.scopedStyle, 'rgb(25, 78, 62)');
  assert.deepEqual(errors, []);
  result.success = true;
} catch (error) {
  result.error = error.message;
  throw error;
} finally {
  await fs.mkdir('reports', { recursive: true });
  await fs.writeFile(`reports/vue-business-${variant}-${result.group}-${Date.now()}.json`, JSON.stringify(result, null, 2));
  console.log(JSON.stringify(result));
  await browser.close();
}
