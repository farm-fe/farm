import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { performance } from 'node:perf_hooks';
import { chromium } from 'playwright-chromium';

const baseURL = process.env.COMMERCE_URL ?? 'http://localhost:4320';
const root = new URL('../', import.meta.url);
const records = [];
const scenarios = [
  { name: 'hmr-leaf', file: 'apps/admin/pages/Components.tsx', from: 'Built to work together', to: 'Commerce leaf changed', selector: '.page-title h1', text: true },
  { name: 'hmr-shared', file: 'packages/ui/index.tsx', from: '<h1>{title}</h1>', to: '<h1>{title} shared changed</h1>', selector: '.page-title h1', expected: 'Built to work together shared changed', original: 'Built to work together', text: true },
  { name: 'hmr-sass', file: 'apps/admin/styles/_tokens.scss', from: '#276d52', to: '#123456', selector: '.eyebrow', property: 'color', expected: 'rgb(18, 52, 86)', original: 'rgb(39, 109, 82)' },
  { name: 'hmr-tailwind', file: 'apps/admin/pages/Components.tsx', from: 'bg-emerald-100', to: 'bg-rose-200', selector: '.rounded-xl', property: 'backgroundColor', expected: 'oklch(0.892 0.058 10.001)', original: 'oklch(0.95 0.052 163.051)' },
  { name: 'hmr-svg', file: 'apps/admin/assets/brand.svg', from: 'rx="9"', to: 'rx="3"', selector: '[aria-label="Interactive SVG"] rect', attribute: 'rx', expected: '3', original: '9' },
  { name: 'hmr-mdx', file: 'apps/admin/content/import-guide.mdx', from: 'Product import playbook', to: 'Commerce MDX changed', selector: '.prose h1', text: true, route: 'help' },
];
const browser = await chromium.launch({ headless: true });
async function observe(page, scenario, changed) {
  const expected = changed ? (scenario.expected ?? scenario.to) : (scenario.original ?? scenario.from);
  await page.waitForFunction(({ selector, property, attribute, expected }) => {
    const node = document.querySelector(selector);
    if (!node) return false;
    const value = property ? getComputedStyle(node)[property] : attribute ? node.getAttribute(attribute) : node.textContent;
    return value === expected;
  }, { ...scenario, expected }, { timeout: 30000 });
}
try {
  for (const scenario of scenarios) {
    const page = await browser.newPage();
    const file = fileURLToPath(new URL(scenario.file, root));
    const original = await fs.readFile(file, 'utf8');
    const errors = [];
    const pendingRequests = new Map();
    page.on('request', request => pendingRequests.set(request, request.url()));
    page.on('requestfinished', request => pendingRequests.delete(request));
    page.on('requestfailed', request => pendingRequests.delete(request));
    page.on('pageerror', error => errors.push(error.message));
    let modified = false;
    let started;
    const record = { group: process.env.COMMERCE_GROUP, baseURL, scenario: scenario.name, timestamp: new Date().toISOString(), success: false };
    try {
      assert.equal(original.split(scenario.from).length, 2, 'Edit anchor must occur exactly once');
      await page.goto(`${baseURL}/#/${scenario.route ?? 'components'}`);
      await observe(page, scenario, false);
      await page.evaluate(() => { window.__commerceHmrDocument = 'unchanged'; });
      if (!scenario.route) await page.getByRole('button', { name: 'Interactive SVG' }).click();
      started = performance.now();
      modified = true;
      await fs.writeFile(file, original.replace(scenario.from, scenario.to));
      await observe(page, scenario, true);
      record.durationMs = performance.now() - started;
      record.sameDocument = await page.evaluate(() => window.__commerceHmrDocument === 'unchanged');
      record.statePreserved = scenario.route ? null : await page.getByText('Icon clicked 1 time', { exact: true }).count() === 1;
      assert.equal(record.sameDocument, true, 'HMR must preserve document');
      if (!scenario.route) assert.equal(record.statePreserved, true, 'HMR must preserve interaction state');
      assert.deepEqual(errors, []);
      record.success = true;
    } catch (error) {
      if (started !== undefined) record.durationMs = performance.now() - started;
      record.error = error.message;
      record.pageErrors = errors;
      record.pendingRequests = [...pendingRequests.values()];
    } finally {
      if (modified) {
        await fs.writeFile(file, original);
        try { await observe(page, scenario, false); record.restored = true; }
        catch (error) { record.success = false; record.restoreError = error.message; }
      }
      if (errors.length) { record.success = false; record.pageErrors = [...errors]; }
      await page.close();
      records.push(record);
      console.log(JSON.stringify(record));
    }
  }
} finally {
  await browser.close();
  if (process.env.COMMERCE_HMR_OUTPUT) await fs.writeFile(process.env.COMMERCE_HMR_OUTPUT, JSON.stringify({ kind: 'functional-preflight', records }, null, 2) + '\n');
}
if (records.some(record => !record.success)) process.exitCode = 1;
