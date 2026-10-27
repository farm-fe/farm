// cspell:words cksum
import test from 'node:test';
import assert from 'node:assert/strict';
import { indexPath, cachedVersions } from './sdk-candidate-registry.mjs';
test('local source replacement uses exact public registry records without invented metadata', () => {
  const row = { name: 'serde', vers: '1.0.0', cksum: 'a'.repeat(64), deps: [], features: {} };
  const bytes = Buffer.from('header\0etag\0' + row.vers + '\0' + JSON.stringify(row) + '\0');
  assert.deepEqual(cachedVersions(bytes, 'serde'), [row]);
  assert.throws(() => cachedVersions(bytes, 'other'));
  assert.equal(indexPath('farmfe_core'), 'fa/rm/farmfe_core');
  assert.equal(indexPath('syn'), '3/s/syn');
  assert.equal(indexPath('Inflector'), 'in/fl/inflector');
  assert.throws(() => indexPath('../escape'));
});
