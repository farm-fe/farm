// Refresh candidate source evidence, never published bytes or version approval.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import { root, npmInventory, cargoInventory } from './v2-release-inventory.mjs';
import { assertNewRustVersion, requiredNativeOwners } from './prepare-cargo-version-proposal.mjs';

export function refreshCandidateSourceInventory(baseline, { npm, cargo, pending, requiredOwners }) {
  assert.equal(baseline.status, 'candidate-not-publication-approval');
  assert.equal(baseline.publication, undefined, 'Cannot refresh a prepared publication; use official version preparation');
  const names = rows => rows.map(row => row.name).sort();
  assert.deepEqual(names(cargo), names(baseline.cargo), 'Cargo release scope changed');
  assert.deepEqual(names(npm.primary), names(baseline.npm.primary), 'npm release scope changed');
  assert.deepEqual(npm.payloads, baseline.npm.payloads, 'Native payload versions require official version preparation');
  for (const row of cargo) {
    const old = baseline.cargo.find(item => item.name === row.name);
    assert.notDeepEqual(row.publish, [], 'Private Cargo package in release scope');
    if (row.version === old.version) {
      assert.equal(row.sourceSha256, old.sourceSha256, `Rust source changed without a version bump: ${row.name}`);
    } else assertNewRustVersion(old.version, row.version);
  }
  const required = new Set(requiredOwners);
  for (const row of npm.primary) {
    const old = baseline.npm.primary.find(item => item.name === row.name);
    assert.equal(row.version, old.version, 'npm versions require official version preparation');
    if (row.sourceSha256 !== old.sourceSha256) required.add(row.name);
  }
  for (const name of required) {
    const owner = npm.primary.find(row => row.name === name);
    const release = pending.find(row => row.name === name);
    assert.ok(owner && release && ['patch', 'minor', 'major'].includes(release.type) &&
      release.oldVersion === owner.version && release.newVersion !== owner.version,
    `Pending Changeset required for changed native/source package: ${name}`);
  }
  // Keep the old preparation receipt untouched. Its digest no longer seals this
  // inventory; pending Changesets must produce new versions and new evidence.
  return { ...baseline, npm, cargo };
}

async function main() {
  assert.equal(fs.realpathSync(process.cwd()), fs.realpathSync(root), 'Run only in the unified repository');
  assert.ok(!fs.existsSync(path.join(root, '.changeset/pre.json')), 'Stable source refresh cannot consume prerelease state');
  const file = path.join(root, 'docs/releases/v2-candidate.json');
  const baseline = JSON.parse(fs.readFileSync(file));
  const metadata = JSON.parse(execFileSync('cargo', ['+nightly-2026-04-10', 'metadata', '--no-deps', '--locked', '--offline', '--format-version', '1'], {
    cwd: root, encoding: 'utf8', maxBuffer: 16 * 1024 ** 2
  }));
  const require = createRequire(fs.realpathSync(path.join(root, 'node_modules/@changesets/cli/package.json')));
  const official = await require('@changesets/get-release-plan').default(root);
  const npm = npmInventory(), cargo = cargoInventory(metadata);
  const changed = cargo.filter(row => row.version !== baseline.cargo.find(old => old.name === row.name)?.version).map(row => row.name);
  const requiredOwners = changed.length ? requiredNativeOwners(baseline, metadata, changed) : [];
  const candidate = refreshCandidateSourceInventory(baseline, { npm, cargo, pending: official.releases, requiredOwners });
  fs.writeFileSync(file, JSON.stringify(candidate, null, 2) + '\n');
  console.log(JSON.stringify({ status: candidate.status, cargoVersionsChanged: changed, pendingNativeOwners: requiredOwners, publicationApproved: false }));
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
