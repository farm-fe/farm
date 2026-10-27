import assert from 'node:assert/strict';
import test from 'node:test';
import { freezeFirstStablePlan, consolidateStableChangelog } from './prepare-v2-stable-versions.mjs';
const plan = { preState: { mode: 'exit' }, changesets: [{ id: 'fix' }], releases: [
  { name: 'core', oldVersion: '2.0.0', newVersion: '3.0.0', type: 'major', changesets: ['fix'] },
  { name: 'private-example', type: 'none' }
] };
const candidate = { npm: { primary: [{ name: 'core', version: '2.0.0' }] } };
const packages = { packages: [{ packageJson: { name: 'core', version: '2.0.0' } }] };
test('previously prepared stable notes are retained without duplicate release headers', () => {
  const source = '# core\n\n## 2.0.0\nNew notes\n\n## 2.0.0\nAuthor notes\n\n## 2.0.0-beta.0\nBeta notes\n';
  const result = consolidateStableChangelog(source, '2.0.0');
  assert.equal(result.match(/^## 2.0.0$/gm).length, 1);
  for (const text of ['New notes', 'Author notes', '## 2.0.0-beta.0', 'Beta notes']) assert.ok(result.includes(text));
});
test('selected packages with no pending changesets still have an explicit stable release entry', () => {
  const result = consolidateStableChangelog('# package\n\n## 2.0.0-beta.0\nOriginal notes\n', '2.0.0');
  assert.ok(result.includes('## 2.0.0\n'));
  assert.ok(result.includes('## 2.0.0-beta.0\nOriginal notes'));
  assert.equal(consolidateStableChangelog(result, '2.0.0'), result);
});
test('first stable consumes changesets without double bump or unrelated private changes', () => {
  const result = freezeFirstStablePlan(plan, candidate, packages);
  assert.equal(result.releases.length, 1);
  assert.equal(result.releases[0].newVersion, '2.0.0');
  assert.deepEqual(result.changesets, plan.changesets);
  assert.equal(plan.releases[0].newVersion, '3.0.0');
});
test('version drift, unexpected public packages and missing exit fail before mutation', () => {
  assert.throws(() => freezeFirstStablePlan({ ...plan, preState: undefined }, candidate, packages));
  assert.throws(() => freezeFirstStablePlan({ ...plan, releases: [...plan.releases, { name: 'unapproved', type: 'patch' }] }, candidate, packages));
  assert.throws(() => freezeFirstStablePlan(plan, candidate, { packages: [{ packageJson: { name: 'core', version: '2.1.0' } }] }));
});
