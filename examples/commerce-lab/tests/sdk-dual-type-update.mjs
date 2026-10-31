import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { captureSourceProvenance } from '../benchmark/source-provenance.mjs';
import { captureNativeProvenance } from '../benchmark/native-provenance.mjs';
import { writeJsonCheckpoint } from '../benchmark/report-checkpoint.mjs';

const sourcePath = 'packages/sdk/src/models.ts';
const original = await fs.readFile(sourcePath, 'utf8');
const anchor = '  stock: number;';
assert.equal(original.split(anchor).length, 2);
const edited = original.replace(anchor, `${anchor}\n  cacheUpdateProbe?: string;`);
const cacheId = `sdk-dual-type-${Date.now()}`;
const filename = `reports/${cacheId}.json`;
const report = { kind: 'sdk-dual-type-update', success: false, cacheId, phases: [],
  scope: 'Actual model edit and guarded restoration, same cache in five phases, fresh same-package ESM/CJS installation per JS/Rust group, strict types and runtime. No controlled performance claim.' };
let changed = false;
async function verify(phase, present) {
  const output = execFileSync(process.execPath, ['tests/sdk-dual-consumer.mjs'], { windowsHide: true, encoding: 'utf8', timeout: 240000,
    env: { ...process.env, COMMERCE_SDK_CACHE_ID: cacheId, COMMERCE_SDK_PHASE: phase, COMMERCE_SDK_EXPECT_PROBE: present ? 'present' : 'absent' } });
  const summary = JSON.parse(output.trim().split(/\r?\n/).at(-1));
  const child = JSON.parse(await fs.readFile(summary.filename, 'utf8'));
  assert.equal(child.success, true);
  assert.equal(child.records.length, 2);
  report.phases.push({ phase, probePresent: present, report: summary.filename, success: true });
  await writeJsonCheckpoint(filename, report);
  console.log(JSON.stringify({ phase, success: true }));
}
try {
  report.sourceBefore = await captureSourceProvenance(); report.nativeBefore = await captureNativeProvenance();
  await verify('baseline', false);
  assert.equal(await fs.readFile(sourcePath, 'utf8'), original);
  changed = true; await fs.writeFile(sourcePath, edited);
  await verify('type-edit', true); await verify('type-edit-warm', true);
} catch (error) { report.error = error.stack; report.output = error.stdout?.toString(); }
finally {
  try {
    if (changed) {
      assert.equal(await fs.readFile(sourcePath, 'utf8'), edited, 'Concurrent model edit: do not overwrite');
      await fs.writeFile(sourcePath, original);
    }
    report.sourceRestored = (await fs.readFile(sourcePath, 'utf8')) === original;
    await verify('restored', false); await verify('restored-warm', false);
    report.sourceAfter = await captureSourceProvenance(); report.nativeAfter = await captureNativeProvenance();
    assert.deepEqual(report.sourceBefore, report.sourceAfter); assert.deepEqual(report.nativeBefore, report.nativeAfter);
    report.success = !report.error && report.phases.length === 5 && report.sourceRestored;
  } catch (error) { report.restoreError = error.stack; }
  await writeJsonCheckpoint(filename, report);
  console.log(JSON.stringify({ filename, success: report.success, error: report.error, restoreError: report.restoreError }));
  if (!report.success) process.exitCode = 1;
}
