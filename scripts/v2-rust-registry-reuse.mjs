import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const firstStableMacros = [
  ['farmfe_macro_cache_item', '2.1.0', 'published-macro-compat.json'],
  ['farmfe_macro_plugin', '2.0.0', 'published-plugin-macro-compat.json'],
];
export const publicRustRegistry = 'registry+https://github.com/rust-lang/crates.io-index';
export function sdkConsumerManifest(plan, text) {
  assert.doesNotMatch(text, /^\s*\[patch|\bpath\s*=/m);
  for (const row of plan.cargo) {
    assert.match(row.name, /^[A-Za-z0-9_-]+$/); assert.match(row.version, /^\d+\.\d+\.\d+$/);
    const pattern = new RegExp('^(\\s*' + row.name + '\\s*=\\s*)"=?[0-9]+\\.[0-9]+\\.[0-9]+"', 'm');
    text = text.replace(pattern, (_, prefix) => prefix + '"=' + row.version + '"');
  }
  return text;
}
export function rustRegistryReuse(plan, { readEvidence = file => JSON.parse(fs.readFileSync(path.join(root, 'docs/release-evidence', file), 'utf8')) } = {}) {
  let records;
  if (['changesets', 'incremental-release'].includes(plan.preparation?.mode)) {
    assert.ok(Array.isArray(plan.publication?.reuseRust), 'Reviewed Changesets Rust reuse set required');
    records = plan.publication.reuseRust;
  } else records = firstStableMacros.map(([name, version, file]) => {
    const evidence = readEvidence(file);
    assert.equal(evidence.status, 'passed', `Published compatibility proof required: ${name}`);
    assert.equal(evidence.workspacePatch, false, `Public registry compatibility proof required: ${name}`);
    assert.equal(evidence.registryPackage?.name, name); assert.equal(evidence.registryPackage.version, version);
    return evidence.registryPackage;
  });
  const seen = new Set();
  return records.map(row => {
    assert.match(row.name ?? '', /^[A-Za-z0-9_-]+$/);
    assert.match(row.version ?? '', /^\d+\.\d+\.\d+$/);
    assert.match(row.sha256 ?? '', /^[a-f0-9]{64}$/);
    assert.ok(!seen.has(row.name), `Duplicate registry reuse: ${row.name}`); seen.add(row.name);
    assert.ok(plan.cargo.some(pkg => pkg.name === row.name && pkg.version === row.version), `Registry reuse is outside exact SDK selection: ${row.name}`);
    return { name: row.name, version: row.version, sha256: row.sha256 };
  });
}
export function copyVerifiedRegistryArchives(records, cache, output) {
  const indexes = fs.existsSync(cache) ? fs.readdirSync(cache).filter(name => name.startsWith('index.crates.io-') && fs.lstatSync(path.join(cache, name)).isDirectory()) : [];
  return records.map(row => {
    const file = `${row.name}-${row.version}.crate`;
    const source = indexes.map(index => path.join(cache, index, file)).find(candidate => fs.existsSync(candidate));
    assert.ok(source, `Missing verified immutable registry archive: ${file}`);
    const stat = fs.lstatSync(source); assert.ok(stat.isFile() && !stat.isSymbolicLink(), `Registry archive must be a regular file: ${file}`);
    const bytes = fs.readFileSync(source);
    assert.equal(crypto.createHash('sha256').update(bytes).digest('hex'), row.sha256, `Immutable registry archive differs: ${file}`);
    fs.copyFileSync(source, path.join(output, file));
    return { ...row, file };
  });
}
