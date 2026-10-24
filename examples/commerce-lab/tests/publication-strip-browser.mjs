import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { spawn } from 'node:child_process';
import { chromium } from 'playwright-chromium';
import { createFixtures } from '../packages/domain/index.mjs';
import { captureSourceProvenance } from '../benchmark/source-provenance.mjs';
import { captureNativeProvenance } from '../benchmark/native-provenance.mjs';
import { writeJsonCheckpoint } from '../benchmark/report-checkpoint.mjs';

const id = `strip-development-${Date.now()}`;
const filename = `reports/${id}.json`;
const report = { kind: 'publication-strip-development', success: false, runtime: process.version,
  scope: 'Development traces and existing production executable JS; sourcemaps intentionally retain original source', cases: [] };
const browser = await chromium.launch({ headless: true });
try {
  report.sourceBefore = await captureSourceProvenance();
  report.nativeBefore = await captureNativeProvenance();
  for (const group of ['js', 'rust', 'mixed']) {
    const files = await fs.readdir(`dist/${group}`, { recursive: true });
    const scripts = files.filter(file => file.endsWith('.js'));
    assert.ok(scripts.length > 0);
    for (const script of scripts) {
      assert.ok(!(await fs.readFile(`dist/${group}/${script}`, 'utf8')).includes('__COMMERCE_DIAGNOSTIC__'),
        `Production executable script retains diagnostic marker: ${group}/${script}`);
    }
    const config = group === 'js' ? 'farm.config.ts' : `farm.${group}.config.ts`;
    const env = { ...process.env, NODE_ENV: 'development', COMMERCE_GROUP: group,
      COMMERCE_CACHE_ID: id, COMMERCE_PORT: '4365' };
    const server = spawn(process.execPath, ['../../packages/cli/bin/farm.mjs', 'start', '--config', config],
      { env, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] });
    let log = '';
    server.stdout.on('data', value => { log += value; });
    server.stderr.on('data', value => { log += value; });
    const context = await browser.newContext();
    const fixture = createFixtures();
    const sku = fixture.products.find(product => product.tenantId === 'north' && product.status === 'published').sku;
    await context.addInitScript(state => localStorage.setItem('farm-commerce-lab-v1', JSON.stringify(state)), fixture);
    const page = await context.newPage();
    const traces = [];
    const errors = [];
    page.on('console', message => { if (message.text().includes('__COMMERCE_DIAGNOSTIC__')) traces.push(message.text()); });
    page.on('pageerror', error => errors.push(error.message));
    try {
      const deadline = Date.now() + 60000;
      while (true) {
        try { if ((await fetch('http://localhost:4365', { signal: AbortSignal.timeout(1000) })).ok) break; } catch { /* owned server startup */ }
        assert.equal(server.exitCode, null, log);
        assert.ok(Date.now() < deadline, log);
        await new Promise(resolve => setTimeout(resolve, 100));
      }
      await page.goto('http://localhost:4365/#/campaigns');
      await page.getByLabel('Campaign name', { exact: true }).fill('Development channel launch');
      await page.getByLabel('Search campaign products', { exact: true }).fill(sku);
      await page.getByLabel(`Campaign product ${sku}`, { exact: true }).check();
      await page.getByRole('button', { name: 'Publish campaign', exact: true }).click();
      await page.locator('article.campaign').getByRole('heading', { name: 'Development channel launch', exact: true }).waitFor();
      assert.deepEqual(traces, ['__COMMERCE_DIAGNOSTIC__ publication-start', '__COMMERCE_DIAGNOSTIC__ publication-success']);
      assert.deepEqual(errors, []);
      report.cases.push({ group, success: true, productionScriptsChecked: scripts.length, developmentTraces: traces });
      await writeJsonCheckpoint(filename, report);
    } finally {
      await context.close();
      if (server.exitCode === null) {
        const exited = new Promise(resolve => server.once('exit', resolve));
        server.kill();
        await exited;
      }
    }
  }
  report.sourceAfter = await captureSourceProvenance();
  report.nativeAfter = await captureNativeProvenance();
  assert.deepEqual(report.sourceAfter, report.sourceBefore);
  assert.deepEqual(report.nativeAfter, report.nativeBefore);
  report.success = true;
} catch (error) {
  report.error = error.stack;
  process.exitCode = 1;
} finally {
  await browser.close();
  await writeJsonCheckpoint(filename, report);
  console.log(JSON.stringify({ filename, success: report.success, cases: report.cases, error: report.error }));
}
