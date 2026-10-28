import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export function lockedPackages(lock) {
  return lock.replaceAll('\r\n', '\n').split('[[package]]').slice(1).map(block => {
    const field = key => block.match(new RegExp(`^${key} = "([^"]+)"`, 'm'))?.[1];
    return { name: field('name'), version: field('version'), source: field('source'), checksum: field('checksum') };
  });
}
export function publicFetchManifest(plan, lock) {
  const own = new Set(plan.cargo.map(row => `${row.name}@${row.version}`));
  const packages = lockedPackages(lock).filter(row => row.source && !own.has(`${row.name}@${row.version}`));
  assert.ok(packages.length);
  for (const pkg of packages) {
    assert.equal(pkg.source, 'registry+https://github.com/rust-lang/crates.io-index', 'Unreviewed non-public SDK dependency');
    assert.match(pkg.name, /^[a-zA-Z0-9_-]+$/); assert.match(pkg.version, /^[a-zA-Z0-9.+-]+$/); assert.match(pkg.checksum, /^[a-f0-9]{64}$/);
  }
  return `[package]\nname = "farm-sdk-public-dependency-cache"\nversion = "0.0.0"\nedition = "2024"\npublish = false\n[workspace]\n[dependencies]\n${packages.map((pkg, index) => `dep${index} = { package = ${JSON.stringify(pkg.name)}, version = ${JSON.stringify('=' + pkg.version)}, default-features = false }`).join('\n')}\n`;
}
export function bindCandidateChecksums(lock, materialization) {
  const expected = [...materialization.selected, ...materialization.reused];
  const seen = new Set();
  const blocks = lock.replaceAll('\r\n', '\n').split('[[package]]');
  for (let index = 1; index < blocks.length; index++) {
    const row = lockedPackages(`[[package]]${blocks[index]}`)[0];
    const record = expected.find(pkg => pkg.name === row.name && pkg.version === row.version);
    if (!record) continue;
    assert.equal(row.source, 'registry+https://github.com/rust-lang/crates.io-index'); assert.match(record.sha256, /^[a-f0-9]{64}$/);
    assert.ok(!seen.has(record.name), 'Duplicate frozen candidate'); seen.add(record.name);
    blocks[index] = blocks[index].replace(/^checksum = "[a-f0-9]{64}"/m, `checksum = "${record.sha256}"`);
  }
  assert.equal(seen.size, expected.length, 'Frozen SDK lock does not cover this candidate; review/update the resolution receipt');
  return blocks.join('[[package]]');
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const root = fileURLToPath(new URL('../', import.meta.url));
  const [mode, destination] = process.argv.slice(2);
  const lock = fs.readFileSync(path.join(root, 'scripts/release-fixtures/sdk-consumer/Cargo.lock'), 'utf8');
  if (mode === 'fetch-recipe') {
    const plan = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
    fs.mkdirSync(destination); fs.writeFileSync(path.join(destination, 'Cargo.toml'), publicFetchManifest(plan, lock), { flag: 'wx' });
    fs.mkdirSync(path.join(destination, 'src')); fs.writeFileSync(path.join(destination, 'src/lib.rs'), '', { flag: 'wx' });
  } else {
    assert.equal(mode, 'bind-candidate');
    const materialization = JSON.parse(fs.readFileSync(path.join(destination, 'materialization.json')));
    fs.writeFileSync(path.join(destination, 'consumer/Cargo.lock'), bindCandidateChecksums(lock, materialization));
  }
}
