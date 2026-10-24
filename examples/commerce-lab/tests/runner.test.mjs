import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

test('runner rejects unknown pipeline before loading any native plugin', () => {
  const script = fileURLToPath(new URL('../scripts/run.mjs', import.meta.url));
  const result = spawnSync(process.execPath, [script, 'build'], { env: { ...process.env, COMMERCE_GROUP: 'typo' }, encoding: 'utf8' });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /Unknown COMMERCE_GROUP/);
});
