// cspell:words cksum
import test from 'node:test';
import assert from 'node:assert/strict';
import { indexPath, cachedVersions, cachedArchiveName, assertLocalCandidateVersion } from './sdk-candidate-registry.mjs';
test('archive build metadata does not become part of the crate name', () => {
  assert.deepEqual(cachedArchiveName('wasi-0.13.3+wasi-0.2.2.crate'), { name: 'wasi', version: '0.13.3+wasi-0.2.2' });
  assert.deepEqual(cachedArchiveName('some-crate-1.0.0-rc.1.crate'), { name: 'some-crate', version: '1.0.0-rc.1' });
  assert.throws(() => cachedArchiveName('../unsafe-1.0.0.crate'));
});
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

test('only PR-only local candidates may shadow public bytes; formal preparation rejects conflicts', () => {
  const publicRow = { name: 'farmfe_core', vers: '2.2.0', cksum: 'a'.repeat(64) };
  const current = { ...publicRow, cksum: 'b'.repeat(64) };
  const pr = { preparation: { mode: 'pr-validation' }, status: 'pr-test-only-not-publication-approval' };
  assertLocalCandidateVersion(pr, publicRow, current);
  for (const mode of ['first-stable', 'changesets', 'incremental-release']) {
    const release = { preparation: { mode } };
    assert.throws(() => assertLocalCandidateVersion(release, publicRow, current), /Immutable public version collision/);
    assertLocalCandidateVersion(release, publicRow, publicRow);
    assertLocalCandidateVersion(release, undefined, current);
  }
  assert.throws(() => assertLocalCandidateVersion({ ...pr, publication: {} }, publicRow, current));
  assert.throws(() => assertLocalCandidateVersion({ ...pr, status: 'ready' }, publicRow, current));
  assert.equal(publicRow.cksum, 'a'.repeat(64));
});
