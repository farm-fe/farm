import test from 'node:test';
import assert from 'node:assert/strict';
import { uploadMetadata, rustPublicationOrder } from './release-rust-metadata.mjs';
const pkg = (name, dependencies = []) => ({ name, version: '2.0.0', source: 'registry+https://github.com/rust-lang/crates.io-index', description: 'SDK crate', dependencies, features: {}, authors: [], readme: null, license: 'MIT' });
const dep = (name, kind = null) => ({ name, req: '=2.0.0', features: ['test'], optional: false, uses_default_features: true, target: 'cfg(windows)', kind, rename: 'renamed' });
test('Cargo upload mapping retains rename, target, build/dev kind and default features', () => {
  const upload = uploadMetadata(pkg('core', [dep('utils', 'build'), dep('tests', 'dev')]));
  assert.equal(upload.deps[0].name, 'utils'); assert.equal(upload.deps[0].explicit_name_in_toml, 'renamed');
  assert.equal(upload.deps[0].target, 'cfg(windows)'); assert.equal(upload.deps[0].default_features, true);
  assert.equal(upload.deps[1].kind, 'dev');
  assert.throws(() => uploadMetadata(pkg('core', [{ ...dep('utils'), path: '/workspace/utils' }])), /Non-registry/);
  assert.throws(() => uploadMetadata(pkg('core', [{ ...dep('utils'), registry: 'https://unreviewed.invalid' }])), /alternate/);
});
test('Rust order includes normal/build prerequisites, excludes dev cycles and rejects real cycles', () => {
  const packages = [pkg('core', [dep('utils')]), pkg('utils', [dep('core', 'dev')])];
  const plan = { cargo: packages.map(({ name, version }) => ({ name, version })) };
  assert.deepEqual(rustPublicationOrder(plan, { packages }).map(row => row.name), ['utils', 'core']);
  packages[1].dependencies[0].kind = 'build';
  assert.throws(() => rustPublicationOrder(plan, { packages }), /cycle/);
});
