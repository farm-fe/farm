import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import test from 'node:test';

test('actual Docs writer command is statically disabled', () => {
  const result = spawnSync('python3', ['-B', 'scripts/docs-pair-protected-writer.py', '--help'], {
    env: { PATH: process.env.PATH }, encoding: 'utf8', timeout: 15_000
  });
  assert.equal(result.status, 1);
  assert.equal(result.stdout, '');
  assert.match(result.stderr, /^Protected Docs publication stopped/);
});

for (const step of ['docs-pair-protected-writer', 'docs-pair-npm-transport']) {
  test(`exercise ${step} with fake network and npm clients`, () => {
    execFileSync('python3', ['-B', `scripts/${step}.test.py`], { timeout: 15_000 });
  });
}
