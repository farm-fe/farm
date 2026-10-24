import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { spawn } from 'node:child_process';
import { chromium } from 'playwright-chromium';
import { createFixtures } from '../packages/domain/index.mjs';
import { captureSourceProvenance } from '../benchmark/source-provenance.mjs';
import { captureNativeProvenance } from '../benchmark/native-provenance.mjs';
import { writeJsonCheckpoint } from '../benchmark/report-checkpoint.mjs';

const scenarios = [
  { name: 'auto-component', file: 'apps/admin/components/ProductImageMetadata.tsx', from: 'Image loaded', to: 'Image ready', mode: 'metadata', selector: '[data-testid="image-dom-status"]' },
  { name: 'modular-ui', file: 'apps/admin/components/ProductBulkActions.tsx', from: 'Clear product selection', to: 'Reset product selection', mode: 'bulk', selector: 'button', textMatch: true },
  { name: 'custom-icon', file: 'apps/admin/assets/icons/inventory.svg', from: 'M3 7h18v14H3z', to: 'M4 7h16v14H4z', mode: 'inventory', selector: '[data-testid="inventory-icon"] path', attribute: 'd' },
  { name: 'image-source', file: 'apps/admin/assets/product-home.svg', from: 'HOME / CANVAS COLLECTION', to: 'HOME / CHANGED COLLECTION', mode: 'metadata', selector: '[data-testid="image-dom-preview"] img', dataImage: true },
];
const id = `extended-hmr-${Date.now()}`;
const filename = `reports/${id}.json`;
const selectedGroups = process.env.COMMERCE_HMR_GROUPS?.split(',') ?? ['js', 'rust', 'mixed'];
const selectedScenarios = scenarios.filter(scenario => !process.env.COMMERCE_HMR_CASE || scenario.name === process.env.COMMERCE_HMR_CASE);
assert.ok(selectedGroups.every(group => ['js', 'rust', 'mixed'].includes(group)));
assert.ok(selectedScenarios.length > 0);
const report = { kind: 'extended-plugin-hmr-correctness', success: false, runtime: process.version, records: [],
  scope: 'Four live-update cases in real Commerce workflows; correctness only, not timings, config restart, compression or arbitrary plugin options' };
const browser = await chromium.launch({ headless: true });
async function observe(page, scenario, expected) {
  await page.waitForFunction(({ selector, attribute, textMatch, dataImage, expected }) => {
    const nodes = [...document.querySelectorAll(selector)];
    return nodes.some(node => {
      if (dataImage) return node.src?.startsWith('data:') && atob(node.src.split(',')[1]).includes(expected);
      return (attribute ? node.getAttribute(attribute) : node.textContent) === expected;
    });
  }, { ...scenario, expected }, { timeout: 15000 });
}
try {
  report.sourceBefore = await captureSourceProvenance();
  report.nativeBefore = await captureNativeProvenance();
  await writeJsonCheckpoint(filename, report);
  report.selectedGroups = selectedGroups; report.selectedScenarios = selectedScenarios.map(scenario => scenario.name);
  for (const group of selectedGroups) {
    const config = group === 'js' ? 'farm.config.ts' : `farm.${group}.config.ts`;
    const server = spawn(process.execPath, ['../../packages/cli/bin/farm.mjs', 'start', '--config', config],
      { env: { ...process.env, NODE_ENV: 'development', COMMERCE_GROUP: group, COMMERCE_PORT: '4366', COMMERCE_CACHE_ID: id },
        windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] });
    let log = '';
    server.stdout.on('data', value => { log += value; }); server.stderr.on('data', value => { log += value; });
    try {
      const deadline = Date.now() + 60000;
      while (true) {
        try { if ((await fetch('http://localhost:4366', { signal: AbortSignal.timeout(1000) })).ok) break; } catch { /* owned server */ }
        assert.equal(server.exitCode, null, log); assert.ok(Date.now() < deadline, log);
        await new Promise(resolve => setTimeout(resolve, 100));
      }
      for (const scenario of selectedScenarios) {
        const original = await fs.readFile(scenario.file, 'utf8');
        const edited = original.replace(scenario.from, scenario.to);
        const context = await browser.newContext();
        await context.addInitScript(state => localStorage.setItem('farm-commerce-lab-v1', JSON.stringify(state)), createFixtures());
        const page = await context.newPage();
        const frames = []; const browserLog = [];
        page.on('websocket', socket => socket.on('framereceived', frame => frames.push(String(frame.payload))));
        page.on('console', message => browserLog.push(message.text()));
        const errors = []; page.on('pageerror', error => errors.push(error.message));
        const record = { group, scenario: scenario.name, success: false, restored: false };
        let changed = false;
        try {
          assert.equal(original.split(scenario.from).length, 2, 'Unique source anchor required');
          await page.goto(`http://localhost:4366/#/${scenario.mode === 'inventory' ? 'inventory' : 'products'}`);
          if (scenario.mode === 'metadata') {
            await page.getByRole('button', { name: 'Preview NORTH-1000', exact: true }).click();
            await page.getByRole('button', { name: 'Inspect image metadata', exact: true }).click();
          } else if (scenario.mode === 'bulk') await page.getByLabel('Select NORTH-1000', { exact: true }).check();
          await observe(page, scenario, scenario.from);
          await page.evaluate(() => { window.__extendedHmrDocument = 'retained'; });
          changed = true;
          record.editStartedAt = Date.now();
          await fs.writeFile(scenario.file, edited);
          await observe(page, scenario, scenario.to);
          record.editObservedAt = Date.now();
          record.sameDocument = await page.evaluate(() => window.__extendedHmrDocument === 'retained');
          assert.equal(record.sameDocument, true);
          if (scenario.mode === 'bulk') assert.equal(await page.getByRole('button', { name: 'Publish selected (1)', exact: true }).isEnabled(), true);
          if (scenario.mode === 'metadata') assert.equal(await page.getByRole('dialog', { name: 'Product media' }).count(), 1);
          assert.deepEqual(errors, []);
          record.success = true;
        } catch (error) { record.error = error.message; record.pageErrors = errors; record.serverLog = log;
          record.frames = frames; record.browserLog = browserLog;
          record.observed = await page.locator(scenario.selector).evaluateAll(nodes => nodes.map(node => ({ text: node.textContent, src: node.getAttribute('src'), d: node.getAttribute('d') })));
        }
        finally {
          if (changed) {
            const current = await fs.readFile(scenario.file, 'utf8');
            assert.equal(current, edited, 'Concurrent source edit: recovery must not overwrite it');
            record.restoreStartedAt = Date.now();
            await fs.writeFile(scenario.file, original);
            try { await observe(page, scenario, scenario.from); record.restored = true; }
            catch (error) { record.success = false; record.restoreError = error.message;
              record.frames = frames; record.browserLog = browserLog; record.serverLog = log;
            }
          }
          await context.close();
          report.records.push(record);
          await writeJsonCheckpoint(filename, report);
          console.log(JSON.stringify({ group, scenario: scenario.name, success: record.success, restored: record.restored, sameDocument: record.sameDocument, error: record.error, restoreError: record.restoreError }));
        }
      }
    } finally {
      if (server.exitCode === null) { const exited = new Promise(resolve => server.once('exit', resolve)); server.kill(); await exited; }
    }
  }
  report.sourceAfter = await captureSourceProvenance(); report.nativeAfter = await captureNativeProvenance();
  assert.deepEqual(report.sourceAfter, report.sourceBefore); assert.deepEqual(report.nativeAfter, report.nativeBefore);
  report.success = report.records.length === selectedGroups.length * selectedScenarios.length && report.records.every(record => record.success && record.restored);
  if (!report.success) process.exitCode = 1;
} catch (error) { report.error = error.stack; process.exitCode = 1; }
finally { await browser.close(); await writeJsonCheckpoint(filename, report); console.log(JSON.stringify({ filename, success: report.success })); }
