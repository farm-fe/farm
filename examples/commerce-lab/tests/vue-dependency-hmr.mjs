import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { chromium } from 'playwright-chromium';
import { monitorBrowserHealth } from './browser-health.mjs';

const file = 'packages/domain/index.mjs';
const original = await fs.readFile(file, 'utf8');
const anchor = 'stock: changes.stock ?? product.stock,';
assert.equal(original.split(anchor).length, 2, 'unique external business edit');
const browser = await chromium.launch({ headless: true });
const page = await browser.newPage();
const health = monitorBrowserHealth(page);
const errors = [];
page.on('pageerror', error => errors.push(error.message));
const variant = process.env.COMMERCE_VUE_VARIANT ?? 'sfc';
assert.ok(['sfc', 'jsx'].includes(variant));
const report = { kind: 'vue-dependency-hmr', variant, group: process.env.COMMERCE_GROUP ?? 'js', success: false, pageErrors: errors };
try {
  await page.goto(process.env.COMMERCE_URL ?? 'http://localhost:4332');
  await page.getByRole('heading', { name: 'Merchant inventory' }).waitFor();
  const documentId = await page.evaluate(() => { window.__vueDependencyDocumentId = crypto.randomUUID(); return window.__vueDependencyDocumentId; });
  await page.getByLabel('Search inventory').fill('NORTH-1000');
  await page.getByRole('button', { name: 'Receive 25 units' }).click();
  await page.getByTestId('stock').filter({ hasText: /^42$/ }).waitFor();
  await fs.writeFile(file, original.replace(anchor, 'stock: (changes.stock ?? product.stock) + 100,'));
  await page.waitForFunction(() => document.querySelector('input[placeholder="SKU, name or category"]')?.value === '', { timeout: 30000 });
  await page.getByLabel('Search inventory').fill('NORTH-1000');
  await page.getByTestId('stock').filter({ hasText: /^17$/ }).waitFor();
  await page.getByRole('button', { name: 'Receive 25 units' }).click();
  await page.getByTestId('stock').filter({ hasText: /^142$/ }).waitFor();
  assert.equal(await page.evaluate(() => window.__vueDependencyDocumentId), documentId, 'dependency HMR must retain the document');
  await fs.writeFile(file, original);
  await page.waitForFunction(() => document.querySelector('input[placeholder="SKU, name or category"]')?.value === '', { timeout: 30000 });
  await page.getByLabel('Search inventory').fill('NORTH-1000');
  await page.getByRole('button', { name: 'Receive 25 units' }).click();
  await page.getByTestId('stock').filter({ hasText: /^42$/ }).waitFor();
  assert.deepEqual(errors, []);
  health.assertHealthy();
  report.success = true;
} catch (error) {
  report.error = error.message;
  throw error;
} finally {
  await fs.writeFile(file, original);
  report.browserHealth = health.evidence;
  health.stop();
  await fs.mkdir('reports', { recursive: true });
  await fs.writeFile(`reports/vue-dependency-hmr-${variant}-${report.group}-${Date.now()}.json`, JSON.stringify(report, null, 2));
  console.log(JSON.stringify(report));
  await browser.close();
}
