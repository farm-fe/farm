import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { npmInventory, root } from './v2-release-inventory.mjs';

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
console.log(JSON.stringify({ sourceInventory: 'consistent', counts: plan.counts, readyToPublish: false, pending: plan.pending }, null, 2));
// A source inventory is deliberately insufficient proof for irreversible publication.
if (process.argv.includes('--publication')) process.exitCode = 1;
