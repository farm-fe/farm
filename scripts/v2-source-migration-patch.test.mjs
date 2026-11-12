import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

const root = fileURLToPath(new URL('../', import.meta.url));
const patchPath = 'docs/vuetify-delivery/vuetify-4.2.5-complete-source.patch';
const patch = readFileSync(new URL(`../${patchPath}`, import.meta.url));
const manifest = JSON.parse(readFileSync(new URL(
  '../docs/vuetify-delivery/manifest.json', import.meta.url
), 'utf8'));
const digest = (bytes) => createHash('sha256').update(bytes).digest('hex');

test('preserved source patch matches its original manifest digest', () => {
  assert.equal(digest(patch), manifest.patchSha256);
});

test('source patch is protected from Git newline conversion', () => {
  const result = spawnSync('git', ['check-attr', 'text', '--', patchPath], {
    cwd: root, encoding: 'utf8'
  });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout.trim(), `${patchPath}: text: unset`);
});

test('normalizing the mixed source lines would invalidate the original digest', () => {
  assert.ok(patch.includes(Buffer.from([13, 10])), 'original inserted CRLF bytes must survive');
  assert.notEqual(digest(patch.toString('utf8').replace(/\r\n/g, '\n')), manifest.patchSha256);
});
