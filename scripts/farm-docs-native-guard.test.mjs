import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {test} from 'node:test';
import {fileURLToPath, pathToFileURL} from 'node:url';
import vm from 'node:vm';

const root = fileURLToPath(new URL('../', import.meta.url));
const workflow = fs.readFileSync(new URL('../.github/workflows/farm-docs.yaml', import.meta.url), 'utf8');
const guard = new URL('./farm-docs-native-guard.mjs', import.meta.url).href;

test('workflow passes native guard options through step output and explicit env', () => {
  const recording = workflow.split('      - name: Record actual native loader paths')[1].split('      - name: Build minimal')[0];
  const envWrite = recording.match(/fs\.appendFileSync\(process\.env\.GITHUB_ENV,`([\s\S]*?)`\);/)[1];
  assert.ok(!envWrite.includes('NODE_OPTIONS'));
  assert.match(recording, /id: native_guard/);
  const code = recording.match(/const nodeOptions=[\s\S]*?fs\.appendFileSync\(process\.env\.GITHUB_OUTPUT,[^\n]+/)[0];
  const writes = [];
  const context = {
    fs: {appendFileSync: (...args) => writes.push(args)},
    path: {resolve: name => path.resolve(root, name)},
    pathToFileURL,
    process: {env: {NODE_OPTIONS: '--trace-warnings', GITHUB_OUTPUT: 'step-output'}}
  };
  vm.runInNewContext(code, context);
  assert.deepEqual(writes, [['step-output', `node_options=--trace-warnings --import=${guard}\n`]]);
  assert.throws(() => vm.runInNewContext(code, {...context, process: {env: {NODE_OPTIONS: 'safe\nforged=value'}}}), /single line/);
  const subsequent = workflow.split('      - name: Build minimal')[1].split('      - name: Preserve')[0];
  const steps = subsequent.split(/\n      - name:/);
  assert.equal(steps.length, 6);
  for (const step of steps) assert.match(step, /NODE_OPTIONS: \$\{\{ steps\.native_guard\.outputs\.node_options \}\}/);
});

test('inherited native guard verifies child processes and rejects changed bytes', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-docs-guard-'));
  try {
    const binary = path.join(directory, 'fixture.bin');
    const baseline = path.join(directory, 'baseline.json');
    fs.writeFileSync(binary, 'fixture bytes');
    fs.writeFileSync(baseline, JSON.stringify({binaries: [{name: 'fixture', path: binary, sha256: crypto.createHash('sha256').update('fixture bytes').digest('hex').toUpperCase()}]}));
    const options = {
      encoding: 'utf8',
      env: {...process.env, NODE_OPTIONS: `--import=${guard}`, CODEX_DOCS_NATIVE_BASELINE: baseline, CODEX_DOCS_NATIVE_GUARD_DIR: path.join(directory, 'receipts')}
    };
    const clean = spawnSync(process.execPath, ['--eval', 'process.stdout.write("verified")'], options);
    assert.equal(clean.status, 0, clean.stderr);
    assert.equal(clean.stdout, 'verified');
    const receipts = fs.readdirSync(path.join(directory, 'receipts')).map(name => JSON.parse(fs.readFileSync(path.join(directory, 'receipts', name))));
    assert.deepEqual(receipts[0].checks.map(check => check.phase), ['before', 'after']);
    const changed = spawnSync(process.execPath, ['--eval', `require('node:fs').writeFileSync(${JSON.stringify(binary)}, 'changed')`], options);
    assert.equal(changed.status, 1);
    assert.match(changed.stderr, /DOCS_NATIVE_DRIFT.*after execution/);
    const rejected = spawnSync(process.execPath, ['--eval', 'process.stdout.write("must not run")'], options);
    assert.notEqual(rejected.status, 0);
    assert.equal(rejected.stdout, '');
    assert.match(rejected.stderr, /DOCS_NATIVE_DRIFT.*before execution/);
  } finally {
    fs.rmSync(directory, {recursive: true, force: true});
  }
});
