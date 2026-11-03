import { releaseCandidatePlanPath } from './v2-release-inventory.mjs';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';
import { sha256 } from './release-archive.mjs';
import { verifyCandidateSeal } from './verify-v2-candidate-seal.mjs';
import { verifySdkConsumer } from './verify-sdk-consumer.mjs';
import { rustPublicationOrder } from './release-rust-metadata.mjs';
import { publicationNpm } from './v2-release-inventory.mjs';

export function assertCompleteSealInputs(plan, npm, sdk) {
  assert.equal(npm.status, 'sealed', 'npm producer is incomplete');
  assert.equal(npm.planSha256, sha256(Buffer.from(JSON.stringify(plan))));
  assert.equal(sdk.planSha256, npm.planSha256); assert.equal(sdk.status, 'passed');
  const identities = values => values.map(row => `${row.name}@${row.version}`).sort();
  const selected = publicationNpm(plan);
  assert.deepEqual(identities(npm.records), identities([...selected.primary, ...selected.payloads]), 'Incomplete or duplicate npm candidate coverage');
  assert.deepEqual(identities(sdk.packages), identities(plan.cargo), 'Incomplete or duplicate Rust candidate coverage');
  for (const key of ['sourceCommit', 'testedCommit', 'testedTree']) assert.match(npm[key], /^[a-f0-9]{40}$/);
}
export function portableSdkFiles(directory, materialization) {
  const files = new Set(['sdk-consumer.json', 'sdk-test.log', 'sdk-metadata.json', 'consumer.Cargo.toml', 'consumer.Cargo.lock', 'materialization.json', 'verification.json']);
  for (const row of [...materialization.selected, ...materialization.reused]) {
    assert.match(row.name, /^[a-zA-Z0-9_-]+$/); assert.match(row.version, /^\d+\.\d+\.\d+$/);
    files.add(`registry/${row.name}-${row.version}.crate`);
  }
  if (materialization.reused.length) {
    files.add('published-macro-license.json');
    const provenance = JSON.parse(fs.readFileSync(path.join(directory, 'published-macro-license.json')));
    for (const row of materialization.reused.filter(row => provenance.packages.some(item => item.name === row.name && item.version === row.version))) files.add(`licenses/${row.name}.LICENSE`);
  }
  const actual = [];
  function visit(relative) {
    const file = path.join(directory, relative), stat = fs.lstatSync(file);
    assert.ok(!stat.isSymbolicLink(), 'Portable SDK symlink refused');
    if (stat.isDirectory()) {
      assert.ok(['', 'registry', 'licenses'].includes(relative), 'Unexpected portable SDK directory');
      for (const name of fs.readdirSync(file)) visit(relative ? `${relative}/${name}` : name);
    } else { assert.ok(stat.isFile() && files.has(relative), `Unreviewed portable SDK member: ${relative}`); actual.push(relative); }
  }
  visit(''); assert.deepEqual(actual.sort(), [...files].sort(), 'Portable SDK members missing');
  return [...files].sort();
}
export function composeReleaseSeal({ root, npmDirectory, sdkDirectory, output }) {
  const read = file => JSON.parse(fs.readFileSync(file));
  const plan = read(releaseCandidatePlanPath(root));
  const npm = read(path.join(npmDirectory, 'receipt.json'));
  const sdk = read(path.join(sdkDirectory, 'sdk-consumer.json'));
  assertCompleteSealInputs(plan, npm, sdk);
  assert.equal(verifyCandidateSeal(plan, npm, npmDirectory, root).candidateArchivesVerified, true);
  verifySdkConsumer(plan, path.join(sdkDirectory, 'sdk-consumer.json'), path.join(root, 'scripts/release-fixtures/sdk-consumer/Cargo.toml'));
  const materialization = read(path.join(sdkDirectory, 'materialization.json'));
  assert.equal(materialization.registry, 'registry', 'Compose only a portable SDK seal');
  const metadata = read(path.join(sdkDirectory, 'sdk-metadata.json'));
  const order = rustPublicationOrder(plan, metadata);
  const portable = portableSdkFiles(sdkDirectory, materialization);
  fs.mkdirSync(output);
  for (const file of portable) {
    fs.mkdirSync(path.dirname(path.join(output, file)), { recursive: true });
    fs.copyFileSync(path.join(sdkDirectory, file), path.join(output, file), fs.constants.COPYFILE_EXCL);
  }
  fs.writeFileSync(path.join(output, 'candidate-plan.json'), JSON.stringify(plan, null, 2) + '\n', { flag: 'wx' });
  const receipt = { ...npm, rust: [], sdkProof: 'sdk-consumer.json', status: 'complete-seal', published: false };
  for (const row of npm.records) {
    assert.equal(path.basename(row.file), row.file);
    fs.copyFileSync(path.join(npmDirectory, row.file), path.join(output, row.file), fs.constants.COPYFILE_EXCL);
  }
  const compatibility = {
    farmfe_macro_cache_item: 'docs/release-evidence/published-macro-compat.json',
    farmfe_macro_plugin: 'docs/release-evidence/published-plugin-macro-compat.json'
  };
  for (const row of order) {
    const source = sdk.packages.find(pkg => pkg.name === row.name && pkg.version === row.version);
    const file = `${row.name}-${row.version}.crate`, metadataFile = `${row.name}-${row.version}.metadata.json`;
    const archive = fs.readFileSync(path.join(sdkDirectory, 'registry', file));
    assert.equal(sha256(archive), source.sha256);
    fs.writeFileSync(path.join(output, file), archive, { flag: 'wx' });
    const bytes = Buffer.from(JSON.stringify(row.metadata)); fs.writeFileSync(path.join(output, metadataFile), bytes, { flag: 'wx' });
    const record = { name: row.name, version: row.version, file, sha256: source.sha256, metadata: metadataFile, metadataSha256: sha256(bytes), dependencies: row.dependencies };
    const reused = plan.publication?.reuseRust?.find(item => item.name === row.name && item.version === row.version);
    const evidence = reused ? plan.preparation?.mode === 'incremental-release' ? 'release-work/release-plan.json' : 'docs/release-evidence/changesets-preparation.json' : compatibility[row.name];
    if (evidence) record.reuseEvidence = { archiveSha256: source.sha256, compatible: true, file: evidence, sha256: sha256(Buffer.from(JSON.stringify(read(path.join(root, evidence))))) };
    receipt.rust.push(record);
  }
  fs.writeFileSync(path.join(output, 'receipt.json'), JSON.stringify(receipt, null, 2), { flag: 'wx' });
  const preflight = execFileSync(process.execPath, [path.join(root, 'scripts/v2-release-preflight.mjs'), '--candidate-seal', path.join(output, 'receipt.json'), '--sdk-proof', path.join(output, 'sdk-consumer.json')], { cwd: root, encoding: 'utf8' });
  const result = JSON.parse(preflight);
  assert.equal(result.candidateArchivesVerified, true); assert.equal(result.externalSdkVerified, true); assert.equal(result.stableVersionStateReady, true);
  fs.writeFileSync(path.join(output, 'prepublication.json'), preflight, { flag: 'wx' });
  return { npm: receipt.records.length, rust: receipt.rust.length, sdk: result.externalSdkVerified, publicRegistryVerified: false };
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [npmDirectory, sdkDirectory, output] = process.argv.slice(2);
  assert.ok(npmDirectory && sdkDirectory && output, 'Expected completed npm directory, portable SDK directory and new output');
  console.log(JSON.stringify(composeReleaseSeal({ root: fileURLToPath(new URL('../', import.meta.url)), npmDirectory: path.resolve(npmDirectory), sdkDirectory: path.resolve(sdkDirectory), output: path.resolve(output) })));
}
