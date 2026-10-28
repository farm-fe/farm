import test from 'node:test';
import assert from 'node:assert/strict';
import { assertCompleteSealInputs } from './compose-v2-release-seal.mjs';
import { sha256 } from './release-archive.mjs';
test('partial, duplicate, stale and missing candidate evidence never forms a complete seal', () => {
  const plan = { npm: { primary: [{ name: 'core', version: '2.0.0' }], payloads: [{ name: 'core-linux', version: '2.0.0' }] }, cargo: [{ name: 'rust-core', version: '2.2.0' }] };
  const npm = { status: 'sealed', planSha256: sha256(Buffer.from(JSON.stringify(plan))), records: [...plan.npm.primary, ...plan.npm.payloads], sourceCommit: 'a'.repeat(40), testedCommit: 'b'.repeat(40), testedTree: 'c'.repeat(40) };
  const sdk = { status: 'passed', planSha256: npm.planSha256, packages: plan.cargo };
  assertCompleteSealInputs(plan, npm, sdk);
  for (const changed of [{ ...npm, status: 'incomplete' }, { ...npm, records: npm.records.slice(1) }, { ...npm, records: [...npm.records, npm.records[0]] }, { ...npm, testedTree: null }]) assert.throws(() => assertCompleteSealInputs(plan, changed, sdk));
  assert.throws(() => assertCompleteSealInputs(plan, npm, { ...sdk, status: 'running' }));
  assert.throws(() => assertCompleteSealInputs(plan, npm, { ...sdk, planSha256: 'old' }));
});
