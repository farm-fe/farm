import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { inspectNpmArchive, sha256, nativePlatformMatches } from './release-archive.mjs';

function sourceLegal(directory) {
  const files = new Map();
  function collect(relative) {
    const file = path.join(directory, relative);
    if (!fs.existsSync(file)) return;
    const stat = fs.lstatSync(file);
    if (stat.isSymbolicLink()) throw new Error('Legal source symlinks are unsupported');
    if (stat.isDirectory()) for (const name of fs.readdirSync(file)) collect(`${relative}/${name}`);
    else if (stat.isFile()) files.set(relative, sha256(fs.readFileSync(file)));
  }
  for (const name of ['LICENSE', 'THIRD_PARTY_NOTICES.md', 'licenses']) collect(name);
  return files;
}

export function verifyCandidateSeal(plan, receipt, directory, sourceRoot = fileURLToPath(new URL('../', import.meta.url))) {
  const expected = new Map([...plan.npm.primary, ...plan.npm.payloads].map(pkg => [pkg.name, pkg]));
  const seen = new Set();
  const records = [];
  const errors = [];
  for (const record of receipt.records ?? []) {
    try {
      const pkg = expected.get(record.name);
      if (!pkg || seen.has(record.name)) throw new Error('Unexpected or duplicate package');
      seen.add(record.name);
      if (record.version !== pkg.version || path.basename(record.file) !== record.file) throw new Error('Package version or archive path mismatch');
      const bytes = fs.readFileSync(path.join(directory, record.file));
      if (sha256(bytes) !== record.sha256) throw new Error('Archive checksum mismatch');
      const result = inspectNpmArchive(bytes, pkg);
      for (const payload of plan.npm.payloads.filter(row => row.owner === pkg.name)) {
        if (result.manifest.optionalDependencies?.[payload.name] !== payload.version) result.issues.push(`Native optional dependency version mismatch: ${payload.name}`);
      }
      const owner = pkg.owner ? plan.npm.primary.find(row => row.name === pkg.owner) : pkg;
      const legal = sourceLegal(path.dirname(path.join(sourceRoot, owner.manifestPath)));
      if (!legal.has('LICENSE')) result.issues.push('Source package license text is missing');
      for (const [file, digest] of legal) if (sha256(result.files.get(`package/${file}`) ?? Buffer.alloc(0)) !== digest) result.issues.push(`Legal bytes absent or changed: ${file}`);
      if (pkg.platform) {
        const independentGenerator = pkg.owner === 'create-farm';
        if (independentGenerator ? record.nativeInterface !== 'node-api-4' : record.abi !== '0.5.4') result.issues.push('Missing or mismatched native interface provenance');
        if (!/^[a-f0-9]{64}$/.test(record.nativeSha256 ?? '')) result.issues.push('Missing native hash provenance');
        const main = result.manifest.main;
        const native = typeof main === 'string' ? result.files.get(`package/${main.replace(/^\.\//, '')}`) : null;
        if (!native || sha256(native) !== record.nativeSha256) result.issues.push('Native payload hash mismatch');
        if (native && !nativePlatformMatches(native, pkg.platform)) result.issues.push('Native file OS/architecture mismatch');
        if (pkg.platform.endsWith('-gnu') && JSON.stringify(result.manifest.libc) !== '["glibc"]') result.issues.push('GNU package lacks explicit glibc selector');
        if (pkg.platform.endsWith('-musl') && JSON.stringify(result.manifest.libc) !== '["musl"]') result.issues.push('musl package lacks explicit libc selector');
        if (!/^[a-f0-9]{40}$/.test(record.producer?.sourceCommit ?? '')) result.issues.push('Missing native producer source identity');
        if (record.producer?.kind === 'local-build') {
          const file = record.producer.receipt;
          if (typeof file !== 'string' || path.basename(file) !== file) throw new Error('Unsafe producer receipt path');
          const source = fs.readFileSync(path.join(directory, file));
          const cohort = JSON.parse(source);
          if (sha256(source) !== record.producer.receiptSha256 || cohort.abi !== record.abi || !cohort.artifacts?.some(row => row.sha256 === record.nativeSha256 && row.producerSha === record.producer.sourceCommit)) result.issues.push('Local producer receipt does not bind this native payload');
        } else if (record.producer?.kind !== 'github-actions' || !record.producer.runId || !record.producer.artifactId) result.issues.push('Missing native build provenance');
      }
      records.push({ name: pkg.name, version: pkg.version, sha256: record.sha256, legal: result.legal, issues: result.issues });
    } catch (error) { errors.push({ name: record.name, error: error.message }); }
  }
  const missing = [...expected.keys()].filter(name => !seen.has(name));
  const invalid = records.filter(record => record.issues.length);
  const ready = missing.length === 0 && errors.length === 0 && invalid.length === 0;
  return { candidateArchivesVerified: ready, expected: expected.size, inspected: records.length, missing, invalid: invalid.map(record => ({ name: record.name, issues: record.issues })), errors, records };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [planPath, receiptPath, outputPath] = process.argv.slice(2);
  if (!planPath || !receiptPath || !outputPath) throw new Error('Usage: node scripts/verify-v2-candidate-seal.mjs plan.json receipt.json output.json');
  const plan = JSON.parse(fs.readFileSync(planPath));
  const receipt = JSON.parse(fs.readFileSync(receiptPath));
  if (receipt.planSha256 !== sha256(Buffer.from(JSON.stringify(plan)))) throw new Error('Receipt belongs to a different candidate plan');
  const result = verifyCandidateSeal(plan, receipt, path.dirname(path.resolve(receiptPath)));
  fs.writeFileSync(outputPath, `${JSON.stringify(result, null, 2)}\n`);
  console.log(JSON.stringify({ candidateArchivesVerified: result.candidateArchivesVerified, inspected: result.inspected, missing: result.missing.length, invalid: result.invalid.length, errors: result.errors.length }));
  if (!result.candidateArchivesVerified) process.exitCode = 1;
}
