import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import { verifySdkConsumer } from './verify-sdk-consumer.mjs';
import { sealSdkConsumer } from './seal-sdk-consumer.mjs';
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');

test('SDK gate independently verifies archive, lock, metadata, fixture and completed three-test evidence', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-sdk-proof-'));
  const archive = Buffer.from('sealed archive');
  const record = { name: 'farmfe_core', version: '2.2.0', sha256: hash(archive) };
  const plan = { cargo: [{ name: record.name, version: record.version }] };
  const planSha256 = hash(JSON.stringify(plan));
  const log = 'test result: ok. 3 passed; 0 failed';
  const manifest = '[workspace]\n[dependencies]\nfarmfe_core = "=2.2.0"\n';
  const proof = path.join(directory, 'sdk-consumer.json');
  const write = (name, value) => fs.writeFileSync(path.join(directory, name), typeof value === 'string' || Buffer.isBuffer(value) ? value : JSON.stringify(value));
  const sdk = { status: 'passed', phase: 'complete', exitCode: 0, sourceMode: 'sealed-local-registry', workspacePatches: false, published: false, sourceCommit: 'a'.repeat(40), planSha256,
    log: 'sdk-test.log', logSha256: hash(log), packages: [{ ...record, source: 'registry+https://github.com/rust-lang/crates.io-index' }] };
  try {
    write('sdk-consumer.json', sdk); write('sdk-test.log', log);
    write('consumer.Cargo.toml', manifest); write('reviewed.toml', manifest);
    write('consumer.Cargo.lock', `[[package]]\nname = "farmfe_core"\nversion = "2.2.0"\nchecksum = "${record.sha256}"\n`);
    write('farmfe_core-2.2.0.crate', archive);
    write('materialization.json', { selected: [record], reused: [], registry: directory, sourceCommit: sdk.sourceCommit, planSha256, workspacePatches: false });
    write('sdk-metadata.json', { workspace_members: ['consumer'], packages: [{ name: record.name, version: record.version, id: 'registry-core', source: sdk.packages[0].source }] });
    assert.equal(verifySdkConsumer(plan, proof, path.join(directory, 'reviewed.toml')).verified, true);
    const portable = path.join(directory, 'portable');
    sealSdkConsumer(plan, proof, path.join(directory, 'reviewed.toml'), portable);
    assert.equal(verifySdkConsumer(plan, path.join(portable, 'sdk-consumer.json'), path.join(directory, 'reviewed.toml')).verified, true);
    assert.equal(JSON.parse(fs.readFileSync(path.join(portable, 'materialization.json'))).registry, 'registry');
    assert.throws(() => sealSdkConsumer(plan, proof, path.join(directory, 'reviewed.toml'), portable), /EEXIST/);
    write('farmfe_core-2.2.0.crate', 'different bytes');
    assert.throws(() => verifySdkConsumer(plan, proof, path.join(directory, 'reviewed.toml')), /registry bytes/);
    write('farmfe_core-2.2.0.crate', archive);
    write('consumer.Cargo.lock', 'unrelated lock');
    assert.throws(() => verifySdkConsumer(plan, proof, path.join(directory, 'reviewed.toml')), /Locked SDK bytes/);
    write('sdk-consumer.json', { ...sdk, status: 'running' });
    assert.throws(() => verifySdkConsumer(plan, proof, path.join(directory, 'reviewed.toml')), /not complete/);
  } finally { fs.rmSync(directory, { recursive: true }); }
});
