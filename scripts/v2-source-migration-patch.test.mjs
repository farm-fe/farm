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


test('unshipped loader archive retains the reviewed source bytes', () => {
  const snapshot = readFileSync(new URL(
    '../docs/source-migration/create-farm-index.js.snapshot', import.meta.url
  ));
  const attribute = spawnSync('git', ['check-attr', 'text', '--',
    'docs/source-migration/create-farm-index.js.snapshot'], { cwd: root, encoding: 'utf8' });
  assert.equal(attribute.status, 0, attribute.stderr);
  assert.equal(attribute.stdout.trim(),
    'docs/source-migration/create-farm-index.js.snapshot: text: unset');
  assert.equal(digest(snapshot), '130263bb5ad77eb3c396d318122025388db9f66f00f1fa57e1cfdd33bbb54e26');
});
