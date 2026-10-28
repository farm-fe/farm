import test from 'node:test';
import assert from 'node:assert/strict';
import { assertCompleteSealInputs, portableSdkFiles } from './compose-v2-release-seal.mjs';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { sha256 } from './release-archive.mjs';

test('portable upload copies only the explicit SDK proof and archive member set', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-portable-sdk-members-'));
  try {
    fs.mkdirSync(path.join(directory, 'registry'));
    const names = ['sdk-consumer.json', 'sdk-test.log', 'sdk-metadata.json', 'consumer.Cargo.toml', 'consumer.Cargo.lock', 'materialization.json', 'verification.json', 'registry/core-2.2.0.crate'];
    for (const name of names) fs.writeFileSync(path.join(directory, name), 'fixture');
    const materialization = { selected: [{ name: 'core', version: '2.2.0' }], reused: [] };
    assert.deepEqual(portableSdkFiles(directory, materialization), names.sort());
    fs.writeFileSync(path.join(directory, '.env'), 'must not upload');
    assert.throws(() => portableSdkFiles(directory, materialization), /Unreviewed portable SDK member/);
  } finally { fs.rmSync(directory, { recursive: true, force: true }); }
});
test('partial, duplicate, stale and missing candidate evidence never forms a complete seal', () => {
  const plan = { npm: { primary: [{ name: 'core', version: '2.0.0' }], payloads: [{ name: 'core-linux', version: '2.0.0' }] }, cargo: [{ name: 'rust-core', version: '2.2.0' }] };
  const npm = { status: 'sealed', planSha256: sha256(Buffer.from(JSON.stringify(plan))), records: [...plan.npm.primary, ...plan.npm.payloads], sourceCommit: 'a'.repeat(40), testedCommit: 'b'.repeat(40), testedTree: 'c'.repeat(40) };
  const sdk = { status: 'passed', planSha256: npm.planSha256, packages: plan.cargo };
  assertCompleteSealInputs(plan, npm, sdk);
  for (const changed of [{ ...npm, status: 'incomplete' }, { ...npm, records: npm.records.slice(1) }, { ...npm, records: [...npm.records, npm.records[0]] }, { ...npm, testedTree: null }]) assert.throws(() => assertCompleteSealInputs(plan, changed, sdk));
  assert.throws(() => assertCompleteSealInputs(plan, npm, { ...sdk, status: 'running' }));
  assert.throws(() => assertCompleteSealInputs(plan, npm, { ...sdk, planSha256: 'old' }));
});
