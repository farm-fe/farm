import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { chromium } from 'playwright-chromium';

const file = 'apps/vue-lab/Inventory.vue';
const original = await fs.readFile(file, 'utf8');
const browser = await chromium.launch({ headless: true });
const page = await browser.newPage();
const errors = [];
page.on('pageerror', error => errors.push(error.message));
const report = { kind: 'vue-hmr-correctness', group: process.env.COMMERCE_GROUP ?? 'js', success: false, cases: [] };
try {
  await page.goto(process.env.COMMERCE_URL ?? 'http://localhost:4332');
  await page.getByRole('heading', { name: 'Merchant inventory' }).waitFor();
  await page.getByLabel('Search inventory').fill('NORTH-1000');
  await page.getByRole('button', { name: 'Receive 25 units' }).click();
  await page.getByTestId('stock').filter({ hasText: /^42$/ }).waitFor();
  const documentId = await page.evaluate(() => { window.__vueDocumentId = crypto.randomUUID(); return window.__vueDocumentId; });
  for (const scenario of [
    { name: 'template', from: '<h1>Merchant inventory</h1>', to: '<h1>Merchant stock workspace</h1>', check: () => page.getByRole('heading', { name: 'Merchant stock workspace' }).waitFor({ timeout: 30000 }), restore: () => page.getByRole('heading', { name: 'Merchant inventory' }).waitFor({ timeout: 30000 }) },
    { name: 'scoped-style', from: 'h1 { color: #194e3e;', to: 'h1 { color: #7a233d;', check: () => page.waitForFunction(() => getComputedStyle(document.querySelector('h1')).color === 'rgb(122, 35, 61)', { timeout: 30000 }), restore: () => page.waitForFunction(() => getComputedStyle(document.querySelector('h1')).color === 'rgb(25, 78, 62)', { timeout: 30000 }) },
  ]) {
    assert.equal(original.split(scenario.from).length, 2, `${scenario.name}: unique edit anchor`);
    const result = { scenario: scenario.name, success: false };
    report.cases.push(result);
    try {
      await fs.writeFile(file, original.replace(scenario.from, scenario.to));
      await scenario.check();
      assert.equal(await page.evaluate(() => window.__vueDocumentId), documentId, 'HMR must preserve the document');
      assert.equal(await page.getByLabel('Search inventory').inputValue(), 'NORTH-1000');
      assert.equal(await page.getByTestId('stock').innerText(), '42', 'template/style HMR must preserve inventory state');
      result.success = true;
    } finally {
      await fs.writeFile(file, original);
      await scenario.restore();
    }
  }
  assert.deepEqual(errors, []);
  report.success = true;
} catch (error) {
  report.error = error.message;
  throw error;
} finally {
  await fs.writeFile(file, original);
  await fs.mkdir('reports', { recursive: true });
  await fs.writeFile(`reports/vue-hmr-${report.group}-${Date.now()}.json`, JSON.stringify(report, null, 2));
  console.log(JSON.stringify(report));
  await browser.close();
}
