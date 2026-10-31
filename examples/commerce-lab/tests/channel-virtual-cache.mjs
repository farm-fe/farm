import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { spawn, execFileSync } from 'node:child_process';
import { chromium } from 'playwright-chromium';
import { captureNativeProvenance } from '../benchmark/native-provenance.mjs';
import { captureSourceProvenance } from '../benchmark/source-provenance.mjs';
import { writeJsonCheckpoint } from '../benchmark/report-checkpoint.mjs';

const cacheId = `channel-rules-${Date.now()}`;
const filename = `reports/${cacheId}.json`;
const report = { kind: 'virtual-channel-config-cache', success: false, cacheId,
  runtime: process.version, scope: 'Windows local artifacts; correctness, not performance or published package acceptance', cases: [] };
const browser = await chromium.launch({ headless: true });
try {
  report.sourceBefore = await captureSourceProvenance();
  report.nativeBefore = await captureNativeProvenance();
  await writeJsonCheckpoint(filename, report);
  for (const group of ['js', 'rust', 'mixed']) {
    // Same cache and output for all three stages: config changes must invalidate stale content.
    for (const version of ['channels-2026-10', 'channels-2026-11', 'channels-2026-10']) {
      const env = { ...process.env, COMMERCE_GROUP: group, COMMERCE_CACHE_ID: cacheId,
        COMMERCE_CHANNEL_RULE_VERSION: version, COMMERCE_PORT: '4363' };
      execFileSync(process.execPath, ['scripts/run.mjs', 'build'], { env, timeout: 180000, stdio: 'pipe', windowsHide: true });
      const config = group === 'js' ? 'farm.config.ts' : `farm.${group}.config.ts`;
      const server = spawn(process.execPath, ['../../packages/cli/bin/farm.mjs', 'preview', '--config', config],
        { env, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] });
      let log = '';
      server.stdout.on('data', value => { log += value; });
      server.stderr.on('data', value => { log += value; });
      const context = await browser.newContext();
      const page = await context.newPage();
      const errors = [];
      page.on('pageerror', error => errors.push(error.message));
      try {
        const deadline = Date.now() + 30000;
        while (true) {
          try {
            if ((await fetch('http://localhost:4363', { signal: AbortSignal.timeout(1000) })).ok) break;
          } catch { /* Await this owned server. */ }
          assert.equal(server.exitCode, null, log);
          assert.ok(Date.now() < deadline, log);
          await new Promise(resolve => setTimeout(resolve, 100));
        }
        await page.goto('http://localhost:4363/#/campaigns');
        const rule = page.getByTestId('channel-rule-version');
        await rule.waitFor();
        assert.equal(await rule.innerText(), `Channel rules: ${version}`);
        assert.deepEqual(await page.getByLabel('Publication channel', { exact: true }).locator('option').allTextContents(),
          ['Store', 'Social', 'Marketplace']);
        assert.deepEqual(errors, []);
        report.cases.push({ group, version, success: true });
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
  }
  report.sourceAfter = await captureSourceProvenance();
  report.nativeAfter = await captureNativeProvenance();
  assert.deepEqual(report.sourceAfter, report.sourceBefore);
  assert.deepEqual(report.nativeAfter, report.nativeBefore);
  report.success = true;
} catch (error) {
  report.error = error.stack;
  report.buildOutput = error.output?.map(value => value?.toString()).filter(Boolean);
  process.exitCode = 1;
} finally {
  await browser.close();
  await writeJsonCheckpoint(filename, report);
  console.log(JSON.stringify({ filename, success: report.success, cases: report.cases, error: report.error }));
}
