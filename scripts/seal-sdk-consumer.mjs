import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { verifySdkConsumer } from './verify-sdk-consumer.mjs';
import { createHash } from 'node:crypto';

export function sealSdkConsumer(plan, proof, fixture, destination) {
  const verified = verifySdkConsumer(plan, proof, fixture);
  const source = path.dirname(path.resolve(proof));
  const materialization = JSON.parse(fs.readFileSync(path.join(source, 'materialization.json')));
  // A separate portable copy preserves all original execution evidence. Only
  // storage locations change; the archived inputs and test log keep their hashes.
  fs.mkdirSync(destination);
  fs.mkdirSync(path.join(destination, 'registry'));
  for (const file of ['sdk-consumer.json', 'sdk-test.log', 'sdk-metadata.json', 'consumer.Cargo.toml', 'consumer.Cargo.lock']) fs.copyFileSync(path.join(source, file), path.join(destination, file), fs.constants.COPYFILE_EXCL);
  for (const record of [...materialization.selected, ...materialization.reused]) {
    const file = `${record.name}-${record.version}.crate`;
    fs.copyFileSync(path.join(path.resolve(source, materialization.registry), file), path.join(destination, 'registry', file), fs.constants.COPYFILE_EXCL);
  }
  if (materialization.reused.length) {
    const root = fileURLToPath(new URL('../', import.meta.url));
    const provenance = JSON.parse(fs.readFileSync(path.join(root, 'docs/release-evidence/published-macro-license.json')));
    fs.mkdirSync(path.join(destination, 'licenses'));
    for (const record of materialization.reused.filter(record => provenance.packages.some(item => item.name === record.name && item.version === record.version))) {
      const license = provenance.packages.find(item => item.name === record.name && item.version === record.version && item.archiveSha256 === record.sha256);
      if (!license) throw new Error(`Missing original license provenance: ${record.name}`);
      const bytes = fs.readFileSync(path.join(root, license.licenseFile));
      if (createHash('sha256').update(bytes).digest('hex') !== license.licenseSha256) throw new Error('Original macro license bytes changed');
      fs.writeFileSync(path.join(destination, 'licenses', `${record.name}.LICENSE`), bytes, { flag: 'wx' });
    }
    fs.writeFileSync(path.join(destination, 'published-macro-license.json'), JSON.stringify(provenance, null, 2), { flag: 'wx' });
  }
  fs.writeFileSync(path.join(destination, 'materialization.json'), JSON.stringify({ ...materialization, registry: 'registry', consumer: 'consumer', storageOnlyRelocated: true }, null, 2), { flag: 'wx' });
  const relocated = verifySdkConsumer(plan, path.join(destination, 'sdk-consumer.json'), fixture);
  if (JSON.stringify(verified) !== JSON.stringify(relocated)) throw new Error('Portable SDK evidence changed');
  fs.writeFileSync(path.join(destination, 'verification.json'), JSON.stringify(relocated, null, 2), { flag: 'wx' });
  return relocated;
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const root = fileURLToPath(new URL('../', import.meta.url));
  const plan = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
  if (!process.argv[2] || !process.argv[3]) throw new Error('Expected completed SDK proof and a new portable output directory');
  console.log(JSON.stringify(sealSdkConsumer(plan, process.argv[2], path.join(root, 'scripts/release-fixtures/sdk-consumer/Cargo.toml'), path.resolve(process.argv[3])), null, 2));
}
