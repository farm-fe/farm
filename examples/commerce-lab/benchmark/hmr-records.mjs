import assert from 'node:assert/strict';

export function validateHmrRecords(records, group) {
  const expected = ['hmr-leaf', 'hmr-shared', 'hmr-sass', 'hmr-tailwind', 'hmr-svg', 'hmr-mdx'];
  assert.equal(records.length, expected.length);
  assert.deepEqual(records.map(record => record.scenario).sort(), [...expected].sort());
  for (const record of records) {
    assert.equal(record.group, group);
    assert.equal(record.success, true, record.error ?? record.restoreError);
    assert.equal(record.sameDocument, true, record.scenario);
    if (record.scenario !== 'hmr-mdx') assert.equal(record.statePreserved, true, record.scenario);
    assert.equal(record.restored, true, record.scenario);
    assert.ok(Number.isFinite(record.durationMs) && record.durationMs >= 0);
  }
}
