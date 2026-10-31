// cspell:words statfs bavail cksum vers
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

export const sha256 = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
export function cachedArchiveName(file) {
  const match = /^([a-zA-Z0-9_-]+)-(\d+\.\d+\.\d+[-+a-zA-Z0-9.]*)\.crate$/.exec(file);
  if (!match) throw new Error('Unexpected cached archive name');
  return { name: match[1], version: match[2] };
}
export function indexPath(name) {
  if (!/^[a-zA-Z0-9_-]+$/.test(name)) throw new Error('Invalid crate name');
  name = name.toLowerCase();
  return name.length === 1 ? `1/${name}` : name.length === 2 ? `2/${name}` : name.length === 3 ? `3/${name[0]}/${name}` : `${name.slice(0, 2)}/${name.slice(2, 4)}/${name}`;
}
export function cachedVersions(bytes, name) {
  return bytes.toString('utf8').split('\0').filter(part => part.startsWith('{')).map(part => {
    const row = JSON.parse(part);
    if (row.name !== name || typeof row.vers !== 'string' || !/^[a-f0-9]{64}$/.test(row.cksum)) throw new Error('Invalid cached public registry metadata');
    return row;
  });
}
function guard(directory) {
  const disk = fs.statfsSync(directory);
  if (os.freemem() < 3.4 * 1024 ** 3 || disk.bavail * disk.bsize < 9.75 * 1024 ** 3) throw new Error('SDK registry materialization resource guard');
}
export function materializeRegistry(cohortDirectory, output, cargoHome = process.env.CARGO_HOME ?? path.join(os.homedir(), '.cargo')) {
  if (fs.existsSync(output)) throw new Error('Preserve existing SDK registry evidence');
  fs.mkdirSync(output, { recursive: true }); guard(output);
  const candidate = JSON.parse(fs.readFileSync(path.join(cohortDirectory, 'receipt.json')));
  if (candidate.status !== 'packaged-not-compiled' || candidate.published !== false) throw new Error('Normal Cargo candidate package receipt required');
  const registry = path.join(output, 'registry'); fs.mkdirSync(path.join(registry, 'index'), { recursive: true });
  const rows = new Map();
  const cache = path.join(cargoHome, 'registry/cache');
  const indices = path.join(cargoHome, 'registry/index');
  let archives = 0, total = 0;
  for (const directory of fs.readdirSync(cache).filter(name => name.startsWith('index.crates.io-'))) {
    for (const entry of fs.readdirSync(path.join(cache, directory), { withFileTypes: true })) {
      if (!entry.isFile() || !entry.name.endsWith('.crate')) continue;
      const { name, version } = cachedArchiveName(entry.name);
      const file = path.join(indices, directory, '.cache', indexPath(name));
      if (!fs.existsSync(file)) continue;
      const metadata = cachedVersions(fs.readFileSync(file), name).find(row => row.vers === version);
      if (!metadata) continue;
      const source = path.join(cache, directory, entry.name);
      const bytes = fs.readFileSync(source); total += bytes.length;
      if (total > 2 * 1024 ** 3 || archives > 5000) throw new Error('Bounded public cache materialization limit exceeded');
      if (sha256(bytes) !== metadata.cksum) throw new Error(`Cached archive checksum mismatch: ${entry.name}`);
      const destination = path.join(registry, entry.name);
      if (!fs.existsSync(destination)) fs.copyFileSync(source, destination, fs.constants.COPYFILE_FICLONE);
      else if (sha256(fs.readFileSync(destination)) !== metadata.cksum) throw new Error('Conflicting immutable cached archive');
      if (!rows.has(name)) rows.set(name, new Map());
      rows.get(name).set(version, metadata); archives++;
      if (archives % 100 === 0) { guard(output); console.log(JSON.stringify({ cachedArchives: archives, cachedBytes: total })); }
    }
  }
  for (const record of candidate.records) {
    const source = path.resolve(cohortDirectory, record.file);
    if (!source.startsWith(path.resolve(cohortDirectory) + path.sep) || sha256(fs.readFileSync(source)) !== record.sha256) throw new Error('Candidate archive path/hash mismatch');
    const file = path.join(cohortDirectory, 'target/package/tmp-registry/index', indexPath(record.name));
    const metadata = fs.readFileSync(file, 'utf8').trim().split('\n').map(JSON.parse).find(row => row.vers === record.version);
    if (!metadata || metadata.name !== record.name || metadata.cksum !== record.sha256) throw new Error('Official Cargo generated index does not bind candidate archive');
    const existing = rows.get(record.name)?.get(record.version);
    if (existing && existing.cksum !== metadata.cksum) throw new Error(`Immutable public version collision: ${record.name}@${record.version}`);
    fs.copyFileSync(source, path.join(registry, `${record.name}-${record.version}.crate`));
    if (!rows.has(record.name)) rows.set(record.name, new Map());
    rows.get(record.name).set(record.version, metadata);
  }
  for (const reused of candidate.reusedRegistry) {
    const file = path.join(registry, `${reused.name}-${reused.version}.crate`);
    if (!fs.existsSync(file) || sha256(fs.readFileSync(file)) !== reused.sha256 || rows.get(reused.name)?.get(reused.version)?.cksum !== reused.sha256) throw new Error('Verified immutable reuse archive is missing from the local source');
  }
  for (const [name, versions] of rows) {
    const file = path.join(registry, 'index', indexPath(name));
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, [...versions.values()].map(row => JSON.stringify(row)).join('\n') + '\n');
  }
  const consumer = path.join(output, 'consumer');
  const recipe = fileURLToPath(new URL('./release-fixtures/sdk-consumer', import.meta.url));
  fs.cpSync(recipe, consumer, { recursive: true });
  fs.mkdirSync(path.join(consumer, '.cargo'));
  fs.writeFileSync(path.join(consumer, '.cargo/config.toml'), `[source.crates-io]\nreplace-with = "farm-candidate"\n[source.farm-candidate]\nlocal-registry = ${JSON.stringify(registry.replaceAll('\\', '/'))}\n[build]\njobs = 1\n`);
  const report = { schemaVersion: 1, sourceCommit: candidate.sourceCommit, planSha256: candidate.planSha256,
    sourceMode: 'sealed-local-registry', workspacePatches: false, consumer, registry,
    cachedArchives: archives, cachedBytes: total, selected: candidate.records, reused: candidate.reusedRegistry,
    compiled: false, published: false };
  fs.writeFileSync(path.join(output, 'materialization.json'), JSON.stringify(report, null, 2));
  return report;
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [cohort, output] = process.argv.slice(2);
  if (!cohort || !output) throw new Error('Expected Cargo cohort and fresh SDK evidence directory');
  const report = materializeRegistry(path.resolve(cohort), path.resolve(output));
  console.log(JSON.stringify({ cachedArchives: report.cachedArchives, selected: report.selected.length, consumer: report.consumer, compiled: false }));
}
