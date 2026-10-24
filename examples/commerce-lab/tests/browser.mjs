import assert from 'node:assert/strict';
import { chromium } from 'playwright-chromium';
import { monitorBrowserHealth } from './browser-health.mjs';
import fs from 'node:fs/promises';

const baseURL = process.env.COMMERCE_URL ?? 'http://localhost:4310';
const browser = await chromium.launch({ headless: true });
const page = await browser.newPage();
const health = monitorBrowserHealth(page);
try {
  await page.goto(baseURL);
  await page.getByTestId('workspace-ready').waitFor();
  await page.getByRole('button', { name: 'Reset demo' }).click();
  await page.getByRole('link', { name: 'Data import', exact: true }).click();
  await page
    .getByLabel('CSV contents')
    .fill(
      'sku,name,price,category\nNEW-100,Weekend tote,39.50,Travel\nNEW-100,Bad price,-2,Home',
    );
  await page.getByRole('button', { name: 'Validate in Worker' }).click();
  await page.getByTestId('import-errors').waitFor();
  assert.match(
    await page.getByTestId('import-errors').innerText(),
    /already exists/,
  );
  assert.equal(
    await page.getByRole('button', { name: 'Import drafts' }).isDisabled(),
    true,
  );
  await page
    .getByLabel('CSV contents')
    .fill('sku,name,price,category\nNEW-100,Weekend tote,39.50,Travel');
  await page.getByRole('button', { name: 'Validate in Worker' }).click();
  await page.getByRole('button', { name: 'Import drafts' }).waitFor();
  await page.getByRole('button', { name: 'Import drafts' }).click();
  await page.getByRole('link', { name: 'Products', exact: true }).click();
  await page.getByLabel('Search products').fill('NEW-100');
  assert.equal(await page.getByTestId('product-row').count(), 1);
  await page.getByRole('button', { name: 'Publish', exact: true }).click();
  await page.getByLabel('Merchant').selectOption('south');
  await page.getByLabel('Search products').fill('NEW-100');
  assert.equal(await page.getByTestId('product-row').count(), 0);
  await page.getByLabel('Merchant').selectOption('north');
  await page.getByRole('link', { name: 'Campaigns', exact: true }).click();
  await page.getByLabel('Campaign name').fill('Weekend launch');
  await page.getByLabel('Discount percent').fill('15');
  await page.getByRole('button', { name: 'Publish campaign' }).click();
  await page.getByRole('alert').getByText(/Invalid campaign product/).waitFor();
  await page.getByLabel('Search campaign products', { exact: true }).fill('NEW-100');
  await page.getByLabel('Campaign product NEW-100', { exact: true }).check();
  await page.getByRole('button', { name: 'Publish campaign' }).click();
  await page.locator('article.campaign').getByRole('heading', { name: 'Weekend launch', exact: true }).waitFor();
  await page.getByRole('link', { name: 'Orders', exact: true }).click();
  await page.getByRole('button', { name: 'Ship', exact: true }).first().click();
  await page.getByRole('link', { name: 'Help center', exact: true }).click();
  await page
    .getByRole('heading', { name: 'Product import playbook' })
    .waitFor();
  await page.getByRole('link', { name: 'Components', exact: true }).click();
  await page.getByRole('button', { name: 'Interactive SVG' }).click();
  await page.getByText('Icon clicked 1 time').waitFor();
  await page.getByLabel('Role').selectOption('viewer');
  await page.getByRole('link', { name: 'Campaigns', exact: true }).click();
  await page.getByText('Your role cannot access this workspace.').waitFor();
  health.assertHealthy();
  console.log(
    JSON.stringify({
      success: true,
      baseURL,
      browserHealth: health.evidence,
      checks: [
        'worker-validation',
        'atomic-import',
        'publish-product',
        'tenant-isolation',
        'campaign',
        'campaign-selection-validation',
        'order-transition',
        'mdx',
        'svg',
        'permissions',
      ],
    }),
  );
} catch (error) {
  const prefix = `reports/browser-smoke-failure-${Date.now()}-${process.pid}`;
  const failure = { error: error.stack, url: page.url(), browserHealth: health.evidence, body: await page.locator('body').innerText({ timeout: 3000 }).catch(() => '[body unavailable]') };
  await fs.mkdir('reports', { recursive: true });
  await fs.writeFile(`${prefix}.json`, JSON.stringify(failure, null, 2));
  await page.screenshot({ path: `${prefix}.png`, fullPage: true }).catch(() => {});
  console.error(JSON.stringify({ failure: `${prefix}.json`, screenshot: `${prefix}.png`, error: error.message }));
  throw error;
} finally {
  health.stop();
  await browser.close();
}
