// Legacy test proofs remain readable, but new PR tests never mint release permission.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import YAML from 'yaml';
for (const name of ['ci', 'rust-test']) test(`${name} has no main test trigger or test-proof signer`, () => {
  const doc = YAML.parse(fs.readFileSync(new URL(`../.github/workflows/${name}.yaml`, import.meta.url), 'utf8'));
  assert.equal(doc.on.push, undefined);
  assert.equal(doc.jobs['sign-ci-bindings'], undefined);
  assert.ok(Object.values(doc.jobs).every(job => !job.uses?.includes('ci-provenance-signer')));
  assert.ok(Object.values(doc.jobs).every(job => job.permissions?.['id-token'] !== 'write'));
});
