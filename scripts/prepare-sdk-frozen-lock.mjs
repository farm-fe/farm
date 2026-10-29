import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';
import { rustRegistryReuse, publicRustRegistry } from './v2-rust-registry-reuse.mjs';

export function lockedPackages(lock) {
  return lock.replaceAll('\r\n', '\n').split('[[package]]').slice(1).map(block => {
    const field = key => block.match(new RegExp(`^${key} = "([^"]+)"`, 'm'))?.[1];
    return { name: field('name'), version: field('version'), source: field('source'), checksum: field('checksum') };
  });
}
export function publicFetchManifest(plan, lock, options) {
  const own = new Set(plan.cargo.map(row => `${row.name}@${row.version}`));
  const locked = lockedPackages(lock);
  const packages = locked.filter(row => row.source && !own.has(`${row.name}@${row.version}`));
  for (const row of rustRegistryReuse(plan, options)) {
    const matches = locked.filter(pkg => pkg.name === row.name && pkg.version === row.version);
    assert.ok(matches.length <= 1, `Ambiguous registry reuse lock: ${row.name}`);
    if (matches.length) {
      assert.equal(matches[0].source, publicRustRegistry, `Reuse lock must use public registry: ${row.name}`);
      assert.equal(matches[0].checksum, row.sha256, `Reuse lock checksum differs: ${row.name}`);
    }
    packages.push({ ...row, source: publicRustRegistry, checksum: row.sha256 });
  }
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
    fs.mkdirSync(destination); // Preserve any existing recipe or failure evidence.
    const receipt = { status: 'recipe-not-started', compiled: false, fetched: false };
    try {
      const plan = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
      receipt.registryReuse = rustRegistryReuse(plan);
      fs.writeFileSync(path.join(destination, 'Cargo.toml'), publicFetchManifest(plan, lock), { flag: 'wx' });
      fs.mkdirSync(path.join(destination, 'src')); fs.writeFileSync(path.join(destination, 'src/lib.rs'), '', { flag: 'wx' });
      receipt.status = 'public-fetch-recipe-created';
    } catch (error) { receipt.status = 'public-fetch-recipe-failed'; receipt.error = error.message.slice(0, 2000); process.exitCode = 1; }
    finally { fs.writeFileSync(path.join(destination, 'receipt.json'), JSON.stringify(receipt, null, 2)); }
  } else {
    assert.equal(mode, 'bind-candidate');
    const materialization = JSON.parse(fs.readFileSync(path.join(destination, 'materialization.json')));
    const plan = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
    if (plan.preparation?.mode === 'changesets') {
      // New crate versions cannot inherit the first-stable frozen resolution.
      // Resolve only against the already verified isolated registry, never paths.
      fs.rmSync(path.join(destination, 'consumer/Cargo.lock'), { force: true });
      execFileSync('cargo', ['+nightly-2026-04-10', 'generate-lockfile', '--offline'], { cwd: path.resolve(destination, 'consumer'), stdio: 'inherit', timeout: 120000 });
    } else fs.writeFileSync(path.join(destination, 'consumer/Cargo.lock'), bindCandidateChecksums(lock, materialization));
  }
}
