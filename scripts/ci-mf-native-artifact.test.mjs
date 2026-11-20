import { selectRemoteA } from '../examples/module-federation-native-probe/interop-config.mjs';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import YAML from 'yaml';
import { nativeTargets } from './native-artifact-inventory.mjs';
import { mfTargets, recordMfNative, consumeMfNative } from './ci-mf-native-artifact.mjs';

function native(abi) {
  const bytes = Buffer.alloc(128);
  if (abi.startsWith('linux')) { bytes.set([127, 69, 76, 70, 2, 1]); bytes.writeUInt16LE(62, 18); }
  if (abi.startsWith('darwin')) { bytes.writeUInt32LE(0xfeedfacf); bytes.writeUInt32LE(0x100000c, 4); }
  if (abi.startsWith('win32')) { bytes.write('MZ'); bytes.writeUInt32LE(64, 60); bytes.write('PE\0\0', 64); bytes.writeUInt16LE(0x8664, 68); }
  return bytes;
}
function fixture(t, abi = 'darwin-arm64') {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'farm-mf-native-')));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const files = ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', '.cargo/config.toml'];
  for (const file of files) { fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true }); fs.writeFileSync(path.join(root, file), file); }
  const source = path.join(root, `rust-plugins/module-federation/npm/${abi}/index.farm`);
  fs.mkdirSync(path.dirname(source), { recursive: true }); fs.writeFileSync(source, native(abi));
  const env = { GITHUB_SHA: 'a'.repeat(40), GITHUB_RUN_ID: '123', GITHUB_RUN_ATTEMPT: '1', GITHUB_REPOSITORY: 'farm-fe/farm', FARM_MF_PRODUCER_ATTEMPT: '1' };
  const options = { root, env, git: command => command === 'diff' ? '' : env.GITHUB_SHA };
  return { root, env, source, options, abi, receipt: path.join(root, `logs/mf-native/${abi}/receipt.json`), bundle: path.join(root, `logs/mf-native/${abi}/index.farm`) };
}
for (const abi of ['linux-x64-gnu', 'darwin-arm64', 'win32-x64-msvc']) test(`same-run private test artifact is verified for ${abi}`, t => {
  const f = fixture(t, abi); const row = recordMfNative(abi, f.options);
  assert.equal(nativeTargets[mfTargets[abi]], abi);
  assert.equal(row.releaseBytesAuthorized, false); assert.equal(row.recipe.profile, 'release');
  fs.writeFileSync(f.source, 'old artifact');
  assert.equal(consumeMfNative(abi, f.options).sha256, row.sha256);
  assert.deepEqual(fs.readFileSync(f.source), native(abi));
});
for (const field of ['sourceSHA', 'runId', 'repository', 'producerAttempt', 'abi', 'target', 'profile', 'sha256', 'bytes', 'releaseBytesAuthorized', 'sourceInputs']) test(`rejects wrong MF receipt ${field}`, t => {
  const f = fixture(t); recordMfNative(f.abi, f.options); const row = JSON.parse(fs.readFileSync(f.receipt));
  if (field === 'target' || field === 'profile') row.recipe[field] = 'wrong';
  else if (field === 'sourceInputs') row.sourceInputs['Cargo.lock'] = 'b'.repeat(64);
  else row[field] = field === 'releaseBytesAuthorized' ? true : field === 'bytes' ? 0 : 'wrong';
  fs.writeFileSync(f.receipt, JSON.stringify(row));
  assert.throws(() => consumeMfNative(f.abi, f.options));
});
test('an Examples retry consumes the bound successful producer attempt, not the current attempt', t => {
  const f = fixture(t); recordMfNative(f.abi, f.options); f.env.GITHUB_RUN_ATTEMPT = '2';
  assert.doesNotThrow(() => consumeMfNative(f.abi, f.options));
  f.env.FARM_MF_PRODUCER_ATTEMPT = '2'; assert.throws(() => consumeMfNative(f.abi, f.options), /attempt/);
});
test('wrong binary bytes or architecture fail closed before installing', t => {
  const f = fixture(t); recordMfNative(f.abi, f.options); fs.writeFileSync(f.bundle, native('linux-x64-gnu'));
  assert.throws(() => consumeMfNative(f.abi, f.options), /bytes|digest/);
  fs.writeFileSync(f.source, native('linux-x64-gnu'));
  assert.throws(() => recordMfNative(f.abi, f.options), /architecture/);
});
test('missing source binding and unsupported targets are rejected', t => {
  const f = fixture(t); assert.throws(() => recordMfNative('../elsewhere', f.options));
  delete f.env.GITHUB_SHA; assert.throws(() => recordMfNative(f.abi, f.options));
});
test('CI builds a separate same-source test artifact and requires its verified consumption', () => {
  const primary = YAML.parse(fs.readFileSync(new URL('../.github/workflows/ci-primary-platform.yaml', import.meta.url), 'utf8'));
  const producer = primary.jobs['mf-native']; assert.ok(producer);
  assert.ok(producer.needs.includes('native-disposition')); assert.equal(producer.runs_on, undefined);
  assert.equal(producer['runs-on'], '${{ inputs.os }}'); assert.ok(!producer['continue-on-error']);
  const build = producer.steps.find(s => s.name === 'Build private MF native test artifact');
  assert.match(build.run, /npm --prefix rust-plugins\/module-federation run build/);
  assert.equal(producer.env.CARGO_TARGET_DIR, undefined, 'runner context is not available in job env');
  assert.equal(build.env.CARGO_TARGET_DIR, '${{ runner.temp }}/mf-native-target');
  assert.match(build.run, /--profile release --locked/); assert.match(build.run, /--target/); assert.match(build.run, /--abi/);
  const upload = producer.steps.find(s => s.id === 'mf-upload');
  assert.match(upload.with.name, /^mf-test-native-/); assert.equal(upload.with['if-no-files-found'], 'error');
  assert.equal(producer.outputs.artifact_id, '${{ steps.mf-upload.outputs.artifact-id }}');
  const consumer = primary.jobs['examples-test']; assert.ok(consumer.needs.includes('mf-native'));
  assert.match(consumer.if, /needs\.mf-native\.result == 'success'/);
  const download = consumer.steps.find(s => s.id === 'download-mf-test');
  assert.equal(download.uses, 'actions/download-artifact@v4.3.0', 'MF artifact IDs require a download action that supports artifact-ids');
  assert.equal(download.with['artifact-ids'], '${{ needs.mf-native.outputs.artifact_id }}');
  const verify = consumer.steps.find(s => s.name === 'Verify and install same-run MF test artifact');
  assert.equal(verify.env.FARM_MF_PRODUCER_ATTEMPT, '${{ needs.mf-native.outputs.run_attempt }}');
  assert.match(verify.run, /ci-mf-native-artifact\.mjs consume/);
  const exampleBuild = consumer.steps.find(s => s.env?.FARM_EXAMPLE_PREBUILT_ONLY);
  assert.equal(exampleBuild.env.FARM_EXAMPLE_PREBUILT_ONLY, '1');
  assert.ok(consumer.steps.indexOf(verify) < consumer.steps.indexOf(exampleBuild));
  const recipe = fs.readFileSync(new URL('./ci-build-rust-artifacts.sh', import.meta.url), 'utf8');
  assert.doesNotMatch(recipe, /module-federation/);
  assert.equal(JSON.parse(fs.readFileSync(new URL('../rust-plugins/module-federation/package.json', import.meta.url))).private, true);
});

test('dirty or different checkout and missing receipt fail closed', t => {
  const f = fixture(t);
  assert.throws(() => recordMfNative(f.abi, { ...f.options, git: command => command === 'diff' ? 'crates/changed.rs' : f.env.GITHUB_SHA }), /committed checkout/);
  assert.throws(() => recordMfNative(f.abi, { ...f.options, git: () => 'b'.repeat(40) }), /current CI checkout/);
  assert.throws(() => consumeMfNative(f.abi, f.options), /ENOENT/);
});
test('artifact and installation redirects are rejected', t => {
  const f = fixture(t); recordMfNative(f.abi, f.options);
  const outside = path.join(f.root, 'outside.farm'); fs.writeFileSync(outside, native(f.abi));
  fs.unlinkSync(f.source); fs.symlinkSync(outside, f.source);
  assert.throws(() => consumeMfNative(f.abi, f.options), /normal|symbolic|symlink|regular|redirect/i);
  fs.unlinkSync(f.bundle); fs.symlinkSync(outside, f.bundle);
  assert.throws(() => consumeMfNative(f.abi, f.options), /normal|symbolic|symlink|regular|redirect/i);
});

for (const paths of [path.posix, path.win32]) test(`interop selects remote-a with ${paths.sep} paths`, () => {
  const configs = ['remote-b', 'remote-a', 'host'].map(name => ({ root: paths.join('fixture', name) }));
  assert.equal(selectRemoteA(configs, paths), configs[1]);
  assert.throws(() => selectRemoteA([configs[0]], paths), /Missing remote-a/);
});
