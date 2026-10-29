// cspell:words raf hashchange
import assert from 'node:assert/strict';
import { monitorBrowserHealth } from '../tests/browser-health.mjs';
export const BUSINESS_CORRECTNESS_PROTOCOL = 'commerce-farm-business-2048-browser-v1';

export async function waitForExtensionCategories(page, health, { expected = 2048, timeout = 15000 } = {}) {
  health.assertHealthy();
  const category = page.getByLabel('Extension category', { exact: true });
  await category.waitFor({ state: 'visible', timeout });
  // React.lazy/hashchange can finish after click resolves. Poll the real DOM,
  // rather than accepting zero options or sleeping for an arbitrary duration.
  await page.waitForFunction(count => {
    const selects = document.querySelectorAll('select[aria-label="Extension category"]');
    return selects.length === 1 && selects[0].options.length === count;
  }, expected, { timeout, polling: 'raf' });
  health.assertHealthy();
  assert.equal(await category.locator('option').count(), expected);
  return category;
}

const boundedText = (value, length = 2000) => String(value ?? '').slice(0, length);
export async function browserFailureEvidence(page, health, { timeoutMs = 1500 } = {}) {
  const browserHealth = {}, counts = {};
  for (const [name, rows] of Object.entries(health.evidence)) {
    counts[name] = rows.length;
    browserHealth[name] = rows.slice(-50).map(row => typeof row === 'string' ? boundedText(row)
      : Object.fromEntries(Object.entries(row).map(([key, value]) => [key, typeof value === 'string' ? boundedText(value) : value && typeof value === 'object' ? boundedText(JSON.stringify(value)) : value])));
  }
  const evidence = { url: null, browserHealth, counts };
  try { evidence.url = boundedText(page.url()); } catch (error) { evidence.urlUnavailable = boundedText(error.message); }
  let timer;
  try {
    assert.ok(Number.isFinite(timeoutMs) && timeoutMs > 0 && timeoutMs <= 5000, 'Bounded diagnostic deadline required');
    const documentTask = Promise.resolve().then(() => page.evaluate(() => ({ title: document.title.slice(0, 500),
      headings: [...document.querySelectorAll('h1,h2,h3')].slice(0, 20).map(node => (node.textContent ?? '').slice(0, 200)),
      body: (document.body?.innerHTML ?? '').slice(0, 8192) })));
    // Promise.race observes late rejection too; do not block the original failure
    // receipt forever on a page whose main thread no longer responds.
    evidence.document = await Promise.race([documentTask, new Promise((_, reject) => { timer = setTimeout(() => reject(new Error('Failure DOM diagnostic deadline exceeded')), timeoutMs); })]);
  } catch (error) { evidence.documentUnavailable = boundedText(error.message); }
  finally { clearTimeout(timer); }
  return evidence;
}

/** Full Farm assertions for each enabled cohort, outside measured build processes. */
export async function verifyBusinessBrowser(page, url) {
  const health = monitorBrowserHealth(page);
  const diagnostics = [];
  const diagnostic = message => { if (message.text().includes('__COMMERCE_DIAGNOSTIC__')) diagnostics.push(message.text()); };
  page.on('console', diagnostic);
  try {
    await page.goto(url);
    await page.getByTestId('workspace-ready').waitFor();
    await page.getByRole('button', { name: 'Reset demo' }).click();
    await page.getByRole('link', { name: 'Components', exact: true }).click();
    const category = await waitForExtensionCategories(page, health);
    for (const id of ['category-001', 'category-1024', 'category-2048']) {
      await category.selectOption(id);
      const section = page.locator(`.extension-${id}`);
      await section.waitFor();
      const field = section.locator('form').getByRole('textbox').first();
      assert.equal(await field.evaluate(input => input.required && input.validity.valueMissing), true);
      await field.fill(`Verified ${id}`);
      await section.getByRole('button', { name: 'Save attributes' }).click();
      await section.locator('form').getByRole('status').filter({ hasText: `Verified ${id}` }).waitFor();
      assert.ok(await section.locator('svg').count() > 0);
      assert.ok(await section.locator('.category-guide table').count() > 0, 'MDX table must survive the production build');
      const style = await section.evaluate(node => ({ display: getComputedStyle(node).display, border: getComputedStyle(node).borderLeftWidth }));
      assert.equal(style.display, 'grid'); assert.equal(style.border, '4px');
      await section.getByLabel('Preview quantity', { exact: true }).fill('5');
      await section.getByLabel('Applied discount', { exact: true }).filter({ hasText: '10%' }).waitFor();
      await section.getByLabel('Promotion total', { exact: true }).filter({ hasText: '$94.96' }).waitFor();
      await section.getByLabel('Delivery destination', { exact: true }).selectOption('export');
      await section.getByLabel('Shipping cost', { exact: true }).filter({ hasText: '$15.00' }).waitFor();
      await section.getByLabel('Preview discount', { exact: true }).fill('21');
      await section.getByRole('alert').filter({ hasText: 'Category discount limit exceeded' }).waitFor();
      await section.getByLabel('Preview discount', { exact: true }).fill('0');
      await section.getByLabel('Preview quantity', { exact: true }).fill('11');
      await section.getByRole('alert').filter({ hasText: 'Insufficient stock after reservation' }).waitFor();
    }
    await page.getByRole('button', { name: 'Interactive SVG' }).click();
    await page.getByText('Icon clicked 1 time', { exact: true }).waitFor();
    await page.getByRole('link', { name: 'Products', exact: true }).click();
    await page.getByRole('button', { name: /^Preview / }).first().click();
    await page.getByRole('button', { name: 'Inspect image metadata' }).click();
    await page.getByTestId('image-dom-status').filter({ hasText: 'Image loaded' }).waitFor();
    await page.getByRole('button', { name: 'Close product preview' }).click();
    await page.getByRole('link', { name: 'Data import', exact: true }).click();
    await page.getByLabel('CSV contents').fill('sku,name,price,category\nPERF-VERIFY,Comparison product,39.50,Travel');
    await page.getByRole('button', { name: 'Validate in Worker' }).click();
    const types = page.getByRole('button', { name: 'Generate TypeScript types' });
    await types.waitFor(); await types.click();
    await page.getByTestId('import-types-code').filter({ hasText: 'ProductImport' }).waitFor();
    await page.getByRole('button', { name: 'Import drafts' }).click();
    await page.getByRole('link', { name: 'Products', exact: true }).click();
    await page.getByLabel('Search products').fill('PERF-VERIFY');
    await page.getByRole('button', { name: 'Publish', exact: true }).click();
    await page.getByRole('link', { name: 'Campaigns', exact: true }).click();
    await page.getByTestId('channel-rule-version').filter({ hasText: 'channels-2026-10' }).waitFor();
    assert.ok(await page.locator('svg').count() > 0, 'Custom virtual icons must render');
    await page.getByLabel('Campaign name').fill('Performance fixture launch');
    await page.getByLabel('Discount percent').fill('15');
    await page.getByLabel('Search campaign products', { exact: true }).fill('PERF-VERIFY');
    await page.getByLabel('Campaign product PERF-VERIFY', { exact: true }).check();
    await page.getByRole('button', { name: 'Publish campaign' }).click();
    await page.locator('article.campaign').getByRole('heading', { name: 'Performance fixture launch', exact: true }).waitFor();
    for (const [name, heading] of [['Overview', 'A clearer view of your business'], ['Orders', 'Every order, accounted for'],
      ['Help center', 'Product import playbook'], ['Inventory', 'Keep the shelves ready'], ['Customers', 'Know the people behind the orders'],
      ['Channels', 'See where growth begins'], ['Campaign rules', 'Clear rules for every launch'], ['Connection', 'Recover with confidence']]) {
      await page.getByRole('link', { name, exact: true }).click();
      await page.getByRole('heading', { name: heading, exact: true }).waitFor();
    }
    health.assertHealthy(); assert.deepEqual(diagnostics, []);
    return { protocol: BUSINESS_CORRECTNESS_PROTOCOL, functional: true, browserHealth: health.evidence,
      checks: ['2048-options', 'first-middle-last-category-interaction', 'required-field', 'SCSS-border', 'Tailwind-grid', 'MDX-table', 'SVG', 'YAML-discount-reservation', 'CSV-tier', 'TSV-shipping', 'image-DOM-auto-component', 'worker-CSV-validation', 'WASM-TypeScript-generation', 'virtual-channels-icons', 'production-strip-during-mock-publication', 'lazy-pages'],
      limitation: 'Three representative category interactions; complete compilation is proven separately by generated source coverage. No real orders or payments.' };
  } catch (original) {
    const error = original instanceof Error ? original : new Error(String(original));
    try { health.assertHealthy(); } catch (healthError) { error.browserHealthFailure = boundedText(healthError.message, 4000); }
    error.browserEvidence = await browserFailureEvidence(page, health);
    throw error;
  } finally { health.stop(); page.off('console', diagnostic); }
}
