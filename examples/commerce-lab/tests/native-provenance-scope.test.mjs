import assert from 'node:assert/strict';
import { test } from 'node:test';
import { nativeProvenanceScope } from '../benchmark/native-provenance.mjs';

test('legacy provenance requires the complete 22-plugin closure', () => {
  const scope = nativeProvenanceScope();
  assert.equal(scope.kind, 'complete-lab');
  assert.equal(scope.plugins.length, 22);
  assert.ok(scope.plugins.includes('vue'));
  assert.ok(scope.plugins.includes('dts'));
  assert.ok(scope.plugins.includes('react-components'));
});

test('only an explicit default JS config uses the audited three-plugin scope', () => {
  assert.deepEqual(nativeProvenanceScope({ groups: ['js'] }), {
    kind: 'default-js-config', groups: ['js'], plugins: ['replace-dirname', 'react', 'worker'],
  });
});

for (const groups of [['rust'], ['mixed'], ['js', 'rust', 'mixed']]) {
  test(`${groups.join('/')} acceptance retains the complete closure`, () => {
    assert.equal(nativeProvenanceScope({ groups }).plugins.length, 22);
  });
}

test('config overrides and independent npm projects cannot select the minimal scope', () => {
  for (const extra of [{ configOverride: '/reviewed/config.ts' }, { npmProject: '/consumer' }, { configOverride: '' }]) {
    assert.equal(nativeProvenanceScope({ groups: ['js'], ...extra }).plugins.length, 22);
  }
});

test('unknown, empty and duplicate groups fail before capturing native artifacts', () => {
  for (const groups of [[], ['unknown'], ['js', 'js'], 'js']) assert.throws(() => nativeProvenanceScope({ groups }), /Invalid native provenance groups/);
});
