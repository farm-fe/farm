import assert from 'node:assert/strict';
import { chromium } from 'playwright-chromium';
import fs from 'node:fs/promises';
import Papa from 'papaparse';

const baseURL = process.env.COMMERCE_URL ?? 'http://localhost:4310';
const browser = await chromium.launch({ headless: true });
try {
  const page = await browser.newPage();
  await page.goto(`${baseURL}/#/import`);
  await page.getByLabel('CSV contents').waitFor();
  const downloaded = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Download template' }).click();
  const download = await downloaded;
  assert.equal(download.suggestedFilename(), 'product-template.csv');
  const csv = await fs.readFile(await download.path(), 'utf8');
  const parsed = Papa.parse(csv, { header: true, skipEmptyLines: true });
  assert.deepEqual(parsed.errors, []);
  assert.deepEqual(parsed.data, [
    { sku: 'IMPORT-001', name: 'Canvas weekend bag', price: '39.50', category: 'Travel' },
    { sku: 'IMPORT-002', name: 'Ceramic espresso set', price: '28.00', category: 'Home' },
  ]);
  await page.getByRole('link', { name: 'Components', exact: true }).click();
  const utility = page.getByText('Ready for your next launch', { exact: true });
  await utility.waitFor();
  const style = await utility.evaluate(element => {
    const css = getComputedStyle(element);
    return { padding: css.padding, radius: css.borderRadius, background: css.backgroundColor, color: css.color };
  });
  assert.deepEqual(style, {
    padding: '24px', radius: '12px',
    background: 'oklch(0.95 0.052 163.051)', color: 'oklch(0.378 0.077 168.94)',
  });
  console.log(JSON.stringify({ success: true, baseURL, checks: ['csv-download', 'tailwind-computed-style'] }));
} finally {
  await browser.close();
}
