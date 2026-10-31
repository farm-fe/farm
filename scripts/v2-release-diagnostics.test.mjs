import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { redactDiagnostic, diagnosticTail, writeBuildFailure, collectReleaseDiagnostics } from './v2-release-diagnostics.mjs';
import { command } from './performance-report.mjs';

test('diagnostic tail is bounded, redacted and cannot emit workflow commands', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-diagnostic-tail-'));
  try {
    const env = { TEST_TOKEN: 'private-value-123' }, file = path.join(root, 'child.log');
    const text = 'old output\n'.repeat(10000) + '::error::private-value-123\npassword=not-env\nBearer private-value\nhttps://user:pw@example.invalid\n';
    fs.writeFileSync(file, text);
    const tail = diagnosticTail(file, env), output = JSON.stringify(tail);
    assert.equal(tail.truncated, true); assert.ok(tail.lines.length <= 80 && output.length < 16000);
    for (const value of ['private-value-123', 'not-env', 'Bearer private-value', 'user:pw', '::error::']) assert.ok(!output.includes(value), value);
    assert.ok(output.includes('REDACTED'));
    assert.ok(!redactDiagnostic('{"password":"hidden-value"}', env).includes('hidden-value'));
    assert.equal(diagnosticTail(path.join(root, 'missing.log')).unavailable, 'Log was not created');
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test('actual failed child reports its cwd, phase and exit code without changing the failure', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-diagnostic-command-'));
  try {
    const log = path.join(root, 'child.log'); let failure;
    await assert.rejects(command(process.execPath, ['-e', 'process.stderr.write("expected child failure"); process.exit(7)'], { phase: 'js-build/fixture', cwd: root, env: process.env, log, timeoutMs: 10000 }), error => { failure = error; return error.exitCode === 7; });
    assert.equal(failure.phase, 'js-build/fixture'); assert.equal(failure.timeoutMs, 10000);
    assert.ok(failure.elapsedMs >= 0);
    const result = writeBuildFailure({ root, phase: 'js-build', name: 'fixture', cwd: root, log, error: failure });
    assert.equal(result.exitCode, 7); assert.equal(result.cwd, root); assert.equal(result.phase, 'js-build');
    assert.ok(result.tail.lines.join('\n').includes('expected child failure'));
    assert.equal(JSON.parse(fs.readFileSync(path.join(root, 'release-diagnostics/js-build-failure.json'))).status, 'failed');
    assert.equal(failure.exitCode, 7);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test('collector reads only known generated receipts and last pack logs, never workspace or environment files', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-diagnostic-allowlist-'));
  try {
    const work = path.join(root, '.release-work'); fs.mkdirSync(path.join(work, 'npm/pack-0'), { recursive: true });
    fs.writeFileSync(path.join(work, 'npm/receipt.json'), JSON.stringify({ status: 'incomplete', errors: ['password=not-env'], records: [{}], environment: { secret: 'must-not-export' } }));
    fs.writeFileSync(path.join(work, 'npm/pack-0/pack.log'), 'pack failed private-value-123');
    fs.writeFileSync(path.join(work, '.env'), 'must-not-export');
    fs.writeFileSync(path.join(work, 'unknown.log'), 'must-not-export');
    const result = collectReleaseDiagnostics(root, work, { TEST_TOKEN: 'private-value-123' });
    const output = fs.readFileSync(path.join(root, 'release-diagnostics/seal-failure.json'), 'utf8');
    assert.equal(result.receipts[0].status, 'incomplete'); assert.equal(result.receipts[0].recordCount, 1);
    assert.ok(result.unavailable.includes('sdk/sdk-consumer.json'));
    for (const value of ['must-not-export', 'private-value-123', 'not-env', 'unknown.log', '.env']) assert.ok(!output.includes(value), value);
    assert.deepEqual(fs.readdirSync(path.join(root, 'release-diagnostics')), ['seal-failure.json']);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test('seal upload uses non-hidden generated output and two explicit sanitized diagnostic files', () => {
  const yaml = fs.readFileSync(new URL('../.github/workflows/v2-candidate-seal.yaml', import.meta.url), 'utf8');
  assert.match(yaml, /path: release-candidate\//);
  assert.match(yaml, /release-diagnostics\/js-build-failure.json/);
  assert.match(yaml, /release-diagnostics\/seal-failure.json/);
  assert.doesNotMatch(yaml, /path: \.release-candidate|\.release-work\/\*\*|include-hidden-files: true/);
});
