import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { chromium } from 'playwright-chromium';
import { monitorBrowserHealth } from './browser-health.mjs';

const variant = process.env.COMMERCE_VUE_VARIANT ?? 'sfc';
assert.ok(['sfc', 'jsx'].includes(variant));
const file = variant === 'jsx' ? 'apps/vue-jsx-lab/Inventory.tsx' : 'apps/vue-lab/Inventory.vue';
const original = await fs.readFile(file, 'utf8');
const anchor = 'stock: stock + 25';
assert.equal(original.split(anchor).length, 2, 'unique script edit anchor');
const browser = await chromium.launch({ headless: true });
const page = await browser.newPage();
const health = monitorBrowserHealth(page);
const errors = [];
page.on('pageerror', error => errors.push(error.message));
const report = { kind: 'vue-script-hmr', variant, group: process.env.COMMERCE_GROUP ?? 'js', success: false, pageErrors: errors };
try {
  await page.goto(process.env.COMMERCE_URL ?? 'http://localhost:4332');
  await page.getByRole('heading', { name: 'Merchant inventory' }).waitFor();
  const documentId = await page.evaluate(() => { window.__vueScriptDocumentId = crypto.randomUUID(); return window.__vueScriptDocumentId; });
  await page.getByLabel('Search inventory').fill('NORTH-1000');
  await page.getByRole('button', { name: 'Receive 25 units' }).click();
  await page.getByTestId('stock').filter({ hasText: /^42$/ }).waitFor();
  await fs.writeFile(file, original.replace(anchor, 'stock: stock + 30'));
  await page.waitForFunction(() => document.querySelector('input[placeholder="SKU, name or category"]')?.value === '', { timeout: 30000 });
  await page.getByLabel('Search inventory').fill('NORTH-1000');
  await page.getByTestId('stock').filter({ hasText: /^17$/ }).waitFor();
  await page.getByRole('button', { name: 'Receive 25 units' }).click();
  await page.getByTestId('stock').filter({ hasText: /^47$/ }).waitFor();
  assert.equal(await page.evaluate(() => window.__vueScriptDocumentId), documentId, 'script HMR must reload the component without replacing the document');
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
  await fs.writeFile(`reports/vue-script-hmr-${variant}-${report.group}-${Date.now()}.json`, JSON.stringify(report, null, 2));
  console.log(JSON.stringify(report));
  await browser.close();
}
