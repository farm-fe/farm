// Adapted from 913dea8cf: use each owning manifest's actual version, including 0.x.
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export async function synchronizeTemplateDependencies(root = process.cwd()) {
  const versions = new Map();
  for (const parent of ['packages', 'rust-plugins', 'js-plugins']) {
    let entries;
    try { entries = await fs.readdir(path.join(root, parent), { withFileTypes: true }); }
    catch (error) { if (error.code === 'ENOENT') continue; throw error; }
    for (const entry of entries) {
      if (!entry.isDirectory()) continue;
      let manifest;
      try { manifest = JSON.parse(await fs.readFile(path.join(root, parent, entry.name, 'package.json'), 'utf8')); }
      catch (error) { if (error.code === 'ENOENT') continue; throw error; }
      if (!manifest.private && manifest.name?.startsWith('@farmfe/')) versions.set(manifest.name, manifest.version);
    }
  }
  const updates = [];
  async function inspect(directory) {
    let entries;
    try { entries = await fs.readdir(directory, { withFileTypes: true }); }
    catch (error) { if (error.code === 'ENOENT') return; throw error; }
    for (const entry of entries) {
      const filename = path.join(directory, entry.name);
      if (entry.isDirectory()) { await inspect(filename); continue; }
      if (!entry.isFile() || entry.name !== 'package.json') continue;
      const manifest = JSON.parse(await fs.readFile(filename, 'utf8'));
      let changed = false;
      for (const section of ['dependencies', 'devDependencies', 'peerDependencies', 'optionalDependencies']) {
        for (const name of Object.keys(manifest[section] ?? {})) {
          if (!name.startsWith('@farmfe/')) continue;
          const version = versions.get(name);
          if (!version) throw new Error(`Unknown template dependency ${name}: ${filename}`);
          const target = `^${version}`;
          if (manifest[section][name] !== target) { manifest[section][name] = target; changed = true; }
        }
      }
      if (changed) updates.push({ filename, manifest });
    }
  }
  await inspect(path.join(root, 'crates/create-farm-rs/templates'));
  await inspect(path.join(root, 'packages/create-farm-plugin/templates'));
  // Validate all templates before writing the first update.
  for (const { filename, manifest } of updates) await fs.writeFile(filename, JSON.stringify(manifest, null, 2) + '\n');
  return updates.map(({ filename }) => path.relative(root, filename));
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  synchronizeTemplateDependencies().then(files => { for (const file of files) console.log(`Updated ${file}`); }).catch(error => { console.error(error.message); process.exitCode = 1; });
}
