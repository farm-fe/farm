import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { npmInventory, root } from './v2-release-inventory.mjs';
import { verifyCandidateSeal } from './verify-v2-candidate-seal.mjs';
import { checkPublishedRustCore } from './check-published-rust-core.mjs';
import { verifySdkConsumer } from './verify-sdk-consumer.mjs';

const plan = JSON.parse(fs.readFileSync(new URL('../docs/releases/v2-candidate.json', import.meta.url), 'utf8'));
assert.deepEqual(npmInventory(), plan.npm, 'Candidate manifests drifted; regenerate and review the release scope');
assert.equal(plan.npm.primary.find(pkg => pkg.name === '@farmfe/js-plugin-vuetify')?.version, '0.0.2');
assert.equal(plan.npm.primary.find(pkg => pkg.name === 'create-farm-plugin')?.version, '2.0.0');
assert.equal(plan.counts.total, plan.npm.primary.length + plan.npm.payloads.length);
assert.ok(!plan.cargo.some(pkg => pkg.version.startsWith('3.')), 'Rust V3 is outside this release scope');
for (const pkg of plan.cargo) {
  assert.notDeepEqual(pkg.publish, [], `${pkg.name} is marked non-publishable`);
  const hash = crypto.createHash('sha256').update(fs.readFileSync(path.join(root, pkg.manifestPath), 'utf8').replaceAll('\r\n', '\n')).digest('hex');
  assert.equal(hash, pkg.manifestSha256, `Cargo candidate manifest drifted: ${pkg.name}`);
}
const hashText = file => crypto.createHash('sha256').update(fs.readFileSync(file, 'utf8').replaceAll('\r\n', '\n')).digest('hex');
const planDigest = crypto.createHash('sha256').update(JSON.stringify(plan)).digest('hex');
const macro = JSON.parse(fs.readFileSync(path.join(root, 'docs/release-evidence/published-macro-compat.json')));
assert.equal(hashText(path.join(root, macro.traitSource)), macro.traitSourceSha256, 'Macro contract source drift');
for (const file of macro.fixtureFiles) assert.equal(hashText(path.join(root, file.file)), file.sha256, 'Macro compatibility fixture drift');
assert.equal(hashText(path.join(root, macro.log)), macro.logSha256, 'Macro test log drift');
assert.match(fs.readFileSync(path.join(root, macro.log), 'utf8'), /test result: ok\. 2 passed; 0 failed/);
const macroLock = fs.readFileSync(path.join(root, 'scripts/release-fixtures/macro-consumer/Cargo.lock'), 'utf8');
assert.match(macroLock, /name = "farmfe_macro_cache_item"\s+version = "2\.1\.0"\s+source = "registry\+https:\/\/github\.com\/rust-lang\/crates\.io-index"\s+checksum = "bf179f567f72a3828f74aff319c1150cdb3335029bfc125ff65774b476e8e502"/);
const pluginMacro = JSON.parse(fs.readFileSync(path.join(root, 'docs/release-evidence/published-plugin-macro-compat.json')));
assert.equal(pluginMacro.registryPackage.name, 'farmfe_macro_plugin');
assert.equal(pluginMacro.registryPackage.version, '2.0.0');
assert.equal(pluginMacro.registryPackage.sha256, '59ac048afc3bfcba0cefe8bbb25e1cdb2e266945094975a95fd9998fd80ae9a2');
for (const file of pluginMacro.fixtureFiles) assert.equal(hashText(path.join(root, file.file)), file.sha256, 'Plugin macro fixture drift');
assert.equal(hashText(path.join(root, pluginMacro.log)), pluginMacro.logSha256);
assert.match(fs.readFileSync(path.join(root, pluginMacro.log), 'utf8'), /test result: ok\. 1 passed; 0 failed/);
const report = { sourceInventory: 'consistent', counts: plan.counts, publishedMacroContractVerified: true, candidateArchivesVerified: false, externalSdkVerified: false, publishedRustCoreVerified: false, readyToPublish: false, pending: [] };
const preFile = path.join(root, '.changeset/pre.json');
const pendingChangesets = fs.readdirSync(path.join(root, '.changeset')).filter(name => name.endsWith('.md') && name !== 'README.md').length;
report.versionState = { prerelease: fs.existsSync(preFile) ? JSON.parse(fs.readFileSync(preFile))?.mode : null, pendingChangesets };
report.stableVersionStateReady = !fs.existsSync(preFile) && pendingChangesets === 0;
if (!report.stableVersionStateReady) report.pending.push('Review beta exit and consume pending changesets in a version PR, then regenerate and re-seal the exact stable version inventory');
const option = key => { const index = process.argv.indexOf(key); return index >= 0 ? process.argv[index + 1] : undefined; };
const sealPath = option('--candidate-seal');
let seal;
if (sealPath) {
  const receipt = JSON.parse(fs.readFileSync(sealPath));
  assert.equal(receipt.planSha256, planDigest, 'Seal belongs to a different release inventory');
  assert.match(receipt.sourceCommit, /^[a-f0-9]{40}$/);
  execFileSync('git', ['diff', '--quiet', receipt.sourceCommit, 'HEAD', '--', 'Cargo.toml', 'Cargo.lock', 'crates', 'rust-ecosystem', 'packages', 'rust-plugins', 'js-plugins', 'patches', 'pnpm-lock.yaml', 'pnpm-workspace.yaml'], { cwd: root });
  seal = verifyCandidateSeal(plan, receipt, path.dirname(path.resolve(sealPath)));
  report.candidateArchivesVerified = seal.candidateArchivesVerified;
  report.archiveCoverage = { inspected: seal.inspected, missing: seal.missing.length, invalid: seal.invalid.length, errors: seal.errors.length };
}
if (!report.candidateArchivesVerified) report.pending.push('Complete verified archives for all 297 selected npm packages, including original legal bytes and native ABI/platform provenance');
const sdkPath = option('--sdk-proof');
if (sdkPath) {
  report.sdkEvidence = verifySdkConsumer(plan, sdkPath, path.join(root, 'scripts/release-fixtures/sdk-consumer/Cargo.toml'));
  execFileSync('git', ['diff', '--quiet', report.sdkEvidence.sourceCommit, 'HEAD', '--', 'Cargo.toml', 'Cargo.lock', 'crates', 'rust-ecosystem', 'patches'], { cwd: root });
  report.externalSdkVerified = true;
}
if (!report.externalSdkVerified) report.pending.push('Independent sealed/public-registry Core, Compiler and Toolkit consumer proof, without workspace patches');
if (process.argv.includes('--publication') && report.candidateArchivesVerified && report.externalSdkVerified && report.stableVersionStateReady) {
  // Live registry verification is a real prerequisite, not a scripted constant.
  // This verifies evidence only; it never publishes or grants publication approval.
  report.publishedRustCore = await checkPublishedRustCore();
  report.publishedRustCoreVerified = true;
}
if (!report.publishedRustCoreVerified) report.pending.push('Before stable npm publication: live published Rust Core 2.2.0 checksum and ABI 0.5.4 verification');
report.readyToPublish = report.candidateArchivesVerified && report.externalSdkVerified && report.publishedRustCoreVerified && report.stableVersionStateReady;
console.log(JSON.stringify(report, null, 2));
if ((process.argv.includes('--publication') && !report.readyToPublish) || (sealPath && !report.candidateArchivesVerified)) process.exitCode = 1;
