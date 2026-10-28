import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { digest, requiredWorkflows, requiredCiJobs } from './v2-auto-release.mjs';
import { rustPublicationOrder } from './release-rust-metadata.mjs';
export function createReleaseIntent(root, plan, metadata, pullRequest = 2329) {
  assert.ok(Number.isSafeInteger(pullRequest) && pullRequest > 0, 'Reviewed version PR number is required');
  const compatibility = { farmfe_macro_cache_item: 'published-macro-compat.json', farmfe_macro_plugin: 'published-plugin-macro-compat.json' };
  const rust = rustPublicationOrder(plan, metadata).map(({ name, version }) => {
    const record = { name, version, action: 'publish' };
    if (compatibility[name]) {
      const evidence = JSON.parse(fs.readFileSync(path.join(root, 'docs/release-evidence', compatibility[name])));
      assert.equal(evidence.status, 'passed'); assert.equal(evidence.registryPackage.name, name); assert.equal(evidence.registryPackage.version, version);
      return { ...record, action: 'reuse', registryChecksum: evidence.registryPackage.sha256, reuseEvidenceFile: `docs/release-evidence/${compatibility[name]}`, reuseEvidenceSha256: digest(JSON.stringify(evidence)) };
    }
    return record;
  });
  const script = 'scripts/verify-published-v2-consumers.mjs';
  return { schemaVersion: 1, pullRequest, baseBranch: 'main', tag: `v${plan.npm.primary.find(pkg => pkg.name === '@farmfe/core').version}`,
    abi: '0.5.4', rustCoreVersion: '2.2.0', candidatePlanSha256: digest(JSON.stringify(plan)), counts: plan.counts,
    requiredWorkflows, requiredCiJobs, performanceRequired: false, seal: { workflowPath: '.github/workflows/ci.yaml' }, candidateSeal: 'receipt.json', sdkProof: 'sdk-consumer.json',
    rust, registryVerification: { script, sha256: digest(fs.readFileSync(path.join(root, script), 'utf8').replaceAll('\r\n', '\n')) }, websiteDeployment: false };
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [metadataFile, destination, prNumber = '2329'] = process.argv.slice(2); assert.ok(metadataFile && destination);
  const root = fileURLToPath(new URL('../', import.meta.url));
  const plan = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
  const intent = createReleaseIntent(root, plan, JSON.parse(fs.readFileSync(metadataFile)), Number(prNumber));
  const text = JSON.stringify(intent, null, 2) + '\n';
  fs.writeFileSync(destination, text, { flag: 'wx' }); console.log(digest(text.replaceAll('\r\n', '\n')));
}
