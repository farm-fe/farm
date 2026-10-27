import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

export const root = fileURLToPath(new URL('../', import.meta.url));
const read = file => JSON.parse(fs.readFileSync(path.join(root, file), 'utf8'));
const digest = value => crypto.createHash('sha256').update(JSON.stringify(value)).digest('hex');

// Candidate scope only. This module never installs, builds, or publishes packages.
export function npmInventory() {
  const primary = [];
  const payloads = [];
  for (const group of ['packages', 'rust-plugins', 'js-plugins']) {
    for (const entry of fs.readdirSync(path.join(root, group)).sort()) {
      const directory = `${group}/${entry}`;
      const manifestPath = `${directory}/package.json`;
      if (!fs.existsSync(path.join(root, manifestPath))) continue;
      const manifest = read(manifestPath);
      if (manifest.private || manifest.name === '@farmfe/js-plugin-copy') continue;
      primary.push({ name: manifest.name, version: manifest.version, manifestPath, manifestSha256: digest(manifest) });
      const npm = path.join(root, directory, 'npm');
      if (!fs.existsSync(npm)) continue;
      for (const platform of fs.readdirSync(npm).sort()) {
        const file = `${directory}/npm/${platform}/package.json`;
        if (!fs.existsSync(path.join(root, file))) continue;
        const pkg = read(file);
        payloads.push({ name: pkg.name, version: manifest.version, owner: manifest.name, platform, manifestPath: file });
      }
    }
  }
  return { primary, payloads, excluded: [{ name: '@farmfe/js-plugin-copy', version: '0.0.4', reason: 'Unmodified independent release; not selected for this V2 candidate.' }] };
}

export function cargoInventory(metadata) {
  const byPath = new Map(metadata.packages.map(pkg => [path.resolve(path.dirname(pkg.manifest_path)), pkg]));
  const selected = new Map();
  function visit(pkg) {
    if (selected.has(pkg.name)) return;
    selected.set(pkg.name, pkg);
    for (const dep of pkg.dependencies) {
      if (!dep.path || dep.kind === 'dev') continue;
      const child = byPath.get(path.resolve(dep.path));
      if (!child) throw new Error(`Missing workspace dependency: ${dep.name}`);
      visit(child);
    }
  }
  for (const name of ['farmfe_core', 'farmfe_compiler', 'farmfe_toolkit', 'farmfe_ecosystem_glob']) {
    const pkg = metadata.packages.find(item => item.name === name);
    if (!pkg) throw new Error(`Missing SDK root: ${name}`);
    visit(pkg);
  }
  return [...selected.values()].map(pkg => ({ name: pkg.name, version: pkg.version, manifestPath: path.relative(root, pkg.manifest_path).replaceAll('\\', '/'), manifestSha256: crypto.createHash('sha256').update(fs.readFileSync(pkg.manifest_path, 'utf8').replaceAll('\r\n', '\n')).digest('hex'), publish: pkg.publish })).sort((a, b) => a.name.localeCompare(b.name));
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [metadataFile, output] = process.argv.slice(2);
  if (!metadataFile || !output) throw new Error('Usage: node scripts/v2-release-inventory.mjs metadata.json output.json');
  const npm = npmInventory();
  const candidate = {
    schemaVersion: 1, status: 'candidate-not-publication-approval', sourceBaseline: '560c55fb984414f16675798912407c8e6afc6434',
    npm, cargo: cargoInventory(JSON.parse(fs.readFileSync(metadataFile, 'utf8'))),
    counts: { primary: npm.primary.length, payloads: npm.payloads.length, total: npm.primary.length + npm.payloads.length },
    pending: ['Final publication SHA and approved package scope', 'Exact-SHA archive hashes, native ABI/platform and license closure receipts', 'Published macro_cache_item 2.1.0 consumer compatibility receipt', 'Public registry installation and independent Docs consumer receipts', 'Protected main review and release decision', 'Verified website rollback deployment target'],
    publicationOrder: ['Rust prerequisites and selected SDK crates; reuse existing versions only after compatibility verification', 'Native npm payloads', 'Runtime, Core, wrappers, CLI, JS plugins and generators', 'Docs and theme together, then independent registry consumer verification', 'Website and release tag only after a separate release decision']
  };
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, `${JSON.stringify(candidate, null, 2)}\n`);
  console.log(JSON.stringify({ counts: candidate.counts, cargo: candidate.cargo.length, status: candidate.status }));
}
