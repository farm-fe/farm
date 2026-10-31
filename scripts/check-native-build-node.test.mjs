import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import semver from 'semver';
import { auditNativeBuildNode } from './check-native-build-node.mjs';

test('native preflight fails on incompatible required engines, missing required packages and empty graphs', () => {
  const packages = [{ name: 'tool', engine: '^22.20 || >=24.12' }];
  assert.equal(auditNativeBuildNode('v20.19.0', { packages, missing: [] }).success, false);
  assert.equal(auditNativeBuildNode('v22.20.0', { packages, missing: [] }).success, true);
  assert.equal(auditNativeBuildNode('v24.12.0', { packages, missing: [{ name: 'optional', optional: true }] }).success, true);
  assert.equal(auditNativeBuildNode('v22.20.0', { packages, missing: [{ name: 'required', optional: false }] }).success, false);
  assert.equal(auditNativeBuildNode('v22.20.0', { packages: [], missing: [] }).success, false);
});

test('native CI host Node declarations satisfy the actual plugin-tools engine', async () => {
  const manifest = JSON.parse(await fs.readFile(new URL('../packages/plugin-tools/package.json', import.meta.url), 'utf8'));
  for (const file of ['rust-build-user.yaml', 'rust-build-release.yaml']) {
    const source = await fs.readFile(new URL(`../.github/workflows/${file}`, import.meta.url), 'utf8');
    const versions = [...source.matchAll(/node-version:\s*['"]?([\d.]+)/g)].map(match => match[1]);
    assert(versions.length > 0, file);
    for (const version of versions) assert(semver.satisfies(semver.coerce(version), manifest.engines.node), `${file}: Node ${version} does not satisfy ${manifest.engines.node}`);
  }
});
