import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';

export function redactDiagnostic(value, env = process.env) {
  let text = String(value).replace(/\u001b\[[0-?]*[ -/]*[@-~]/g, '');
  for (const [name, secret] of Object.entries(env)) if (/TOKEN|SECRET|PASSWORD|CREDENTIAL|PRIVATE_KEY|ACCESS_KEY|AUTHORIZATION/i.test(name) && typeof secret === 'string') {
    for (const part of [secret, ...secret.split(/\r?\n/)].filter(part => part.length >= 6)) for (const form of [part, encodeURIComponent(part)]) text = text.split(form).join('[REDACTED]');
  }
  return text.replace(/https?:\/\/[^\s/@:]+:[^\s/@]+@/gi, 'https://[REDACTED]@')
    .replace(/\b(?:Bearer|Basic)\s+[A-Za-z0-9+/_=.-]+/gi, '[REDACTED_AUTH]')
    .replace(/\b(?:gh(?:p|o|u|s|r)_[A-Za-z0-9_]+|github_pat_[A-Za-z0-9_]+|npm_[A-Za-z0-9]+)\b/g, '[REDACTED_TOKEN]')
    .replace(/\b((?:[\w-]*(?:token|secret|password|authorization|credential|private_key|access_key)[\w-]*))["']?\s*[:=]\s*(?:"[^"]*"|'[^']*'|[^\s,;]+)/gi, '$1=[REDACTED]')
    .replaceAll('::', ': :').replace(/[\u0000-\u0008\u000b-\u001f\u007f]/g, '');
}
export function diagnosticTail(file, env = process.env) {
  if (!fs.existsSync(file)) return { unavailable: 'Log was not created', lines: [] };
  assert.ok(!fs.lstatSync(file).isSymbolicLink() && fs.statSync(file).isFile(), 'Diagnostic log must be a regular generated file');
  const fd = fs.openSync(file, 'r');
  try {
    const size = fs.fstatSync(fd).size, length = Math.min(size, 65536), bytes = Buffer.alloc(length);
    fs.readSync(fd, bytes, 0, length, size - length);
    const lines = redactDiagnostic(bytes.toString('utf8'), env).slice(-12000).split(/\r?\n/).slice(-80);
    return { truncated: size > length || lines.length === 80, lines };
  } finally { fs.closeSync(fd); }
}
function diagnosticDirectory(root) {
  const directory = path.join(root, 'release-diagnostics');
  if (fs.existsSync(directory)) assert.ok(!fs.lstatSync(directory).isSymbolicLink() && fs.statSync(directory).isDirectory());
  else fs.mkdirSync(directory);
  return directory;
}
export function writeBuildFailure({ root, phase, name, cwd, log, error, env = process.env }) {
  const record = { schemaVersion: 1, status: 'failed', sourceCommit: env.GITHUB_SHA ?? null, phase, package: name, cwd,
    exitCode: Number.isInteger(error.exitCode) ? error.exitCode : null, signal: error.signal ?? null,
    error: redactDiagnostic(error.message, env), timedOut: error.timedOut === true,
    cleanupConfirmed: typeof error.cleanupConfirmed === 'boolean' ? error.cleanupConfirmed : null,
    tail: diagnosticTail(log, env) };
  const destination = path.join(diagnosticDirectory(root), 'js-build-failure.json');
  fs.writeFileSync(destination, JSON.stringify(record, null, 2) + '\n', { flag: 'wx' });
  console.error('[candidate-js failure] ' + JSON.stringify(record));
  return record;
}
export function collectReleaseDiagnostics(root, work, env = process.env) {
  const result = { schemaVersion: 1, status: 'failed-or-incomplete', sourceCommit: env.GITHUB_SHA ?? null, receipts: [], logs: [], unavailable: [] };
  const safeFile = relative => {
    let file = work;
    for (const part of relative.split('/')) { file = path.join(file, part); if (fs.existsSync(file)) assert.ok(!fs.lstatSync(file).isSymbolicLink(), 'Diagnostic symlink refused'); }
    return file;
  };
  if (fs.existsSync(work)) assert.ok(!fs.lstatSync(work).isSymbolicLink() && fs.statSync(work).isDirectory());
  for (const relative of ['native-assembly.json', 'npm/receipt.json', 'npm/verification.json', 'rust/receipt.json', 'sdk/sdk-consumer.json']) {
    const file = safeFile(relative);
    if (!fs.existsSync(file)) { result.unavailable.push(relative); continue; }
    try {
      assert.ok(fs.statSync(file).isFile() && fs.statSync(file).size <= 4 * 1024 * 1024);
      const data = JSON.parse(fs.readFileSync(file, 'utf8')), summary = { file: relative };
      for (const key of ['status', 'phase', 'sourceCommit', 'exitCode', 'candidateArchivesVerified', 'error', 'errors']) {
        const value = data[key];
        if (value === undefined) continue;
        if (typeof value === 'string') summary[key] = redactDiagnostic(value, env).slice(0, 12000);
        else if (value === null || ['number', 'boolean'].includes(typeof value)) summary[key] = value;
        else if (Array.isArray(value)) summary[key] = value.slice(0, 20).map(item => redactDiagnostic(typeof item === 'string' ? item : JSON.stringify(item), env).slice(0, 1000));
        else summary[key] = 'Unsupported diagnostic field type';
      }
      if (Array.isArray(data.records)) summary.recordCount = data.records.length;
      result.receipts.push(summary);
    } catch (error) { result.unavailable.push(`${relative}: ${redactDiagnostic(error.message, env)}`); }
  }
  const logs = ['rust/stdout.txt', 'rust/diagnostic.txt', 'sdk/sdk-test.log'];
  const npm = path.join(work, 'npm');
  if (fs.existsSync(npm)) {
    assert.ok(!fs.lstatSync(npm).isSymbolicLink());
    const stages = fs.readdirSync(npm).filter(name => /^pack-\d+$/.test(name)).sort((a, b) => Number(a.slice(5)) - Number(b.slice(5)));
    // Preserve the last three actual pack stages, never scan arbitrary files.
    for (const name of stages.slice(-3)) logs.push(`npm/${name}/pack.log`);
  }
  for (const relative of logs) {
    const file = safeFile(relative);
    if (fs.existsSync(file)) result.logs.push({ file: relative, ...diagnosticTail(file, env) });
  }
  const file = path.join(diagnosticDirectory(root), 'seal-failure.json');
  fs.writeFileSync(file, JSON.stringify(result, null, 2) + '\n', { flag: 'wx' });
  console.error('[candidate seal diagnostics] ' + JSON.stringify({ status: result.status, sourceCommit: result.sourceCommit, unavailable: result.unavailable }));
  for (const receipt of result.receipts) console.error('[candidate seal receipt] ' + JSON.stringify(receipt));
  for (const log of result.logs) console.error('[candidate seal log tail] ' + JSON.stringify(log));
  return result;
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const root = path.resolve(fileURLToPath(new URL('../', import.meta.url))), work = path.resolve(process.argv[2] ?? '.release-work');
  assert.ok(work.startsWith(root + path.sep) || work === path.join(root, '.release-work'), 'Diagnostic source must stay in the checkout');
  collectReleaseDiagnostics(root, work);
}
