import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { changesetStatus } from './check-changeset-status.mjs';

test('pending, empty and prerelease are explicit states', t => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-changeset-state-'));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  fs.mkdirSync(path.join(root, '.changeset'));
  assert.equal(changesetStatus(root).state, 'empty');
  fs.writeFileSync(path.join(root, '.changeset/a.md'), '---\n---\nChange');
  assert.equal(changesetStatus(root).state, 'pending');
  for (const mode of ['pre', 'exit']) {
    fs.writeFileSync(path.join(root, '.changeset/pre.json'), JSON.stringify({ mode, changesets: ['a'] }));
    assert.equal(changesetStatus(root).state, 'pending');
  }
});

test('missing, malformed and invalid prerelease fail with error, never empty', t => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-changeset-error-'));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const check = () => {
    const result = spawnSync(process.execPath, [fileURLToPath(new URL('./check-changeset-status.mjs', import.meta.url))], { cwd: root, encoding: 'utf8' });
    assert.equal(result.status, 2);
    assert.equal(JSON.parse(result.stderr).state, 'error');
  };
  check();
  fs.mkdirSync(path.join(root, '.changeset'));
  for (const value of ['{', '{}', '{"mode":"pre","changesets":false}']) {
    fs.writeFileSync(path.join(root, '.changeset/pre.json'), value);
    check();
  }
});
