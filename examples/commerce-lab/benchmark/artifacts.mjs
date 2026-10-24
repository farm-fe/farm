import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { gunzipSync } from 'node:zlib';

async function files(directory, relative = '') {
  const result = [];
  for (const entry of await fs.readdir(path.join(directory, relative), { withFileTypes: true })) {
    const name = path.join(relative, entry.name);
    if (entry.isDirectory()) result.push(...await files(directory, name));
    else if (entry.isFile()) result.push(name);
  }
  return result.sort();
}

export async function verifyCompression(directory) {
  const names = await files(directory);
  const available = new Set(names);
  let verifiedGzipFiles = 0;
  let sourceBytes = 0;
  let gzipBytes = 0;
  for (const name of names) {
    if (name.endsWith('.gz')) {
      const original = name.slice(0, -3);
      if (!available.has(original)) throw new Error(`Orphan gzip: ${name}`);
      const compressed = await fs.readFile(path.join(directory, name));
      const source = await fs.readFile(path.join(directory, original));
      if (!gunzipSync(compressed).equals(source)) throw new Error(`Gzip differs from original: ${name}`);
      verifiedGzipFiles++;
      sourceBytes += source.length;
      gzipBytes += compressed.length;
    } else if (/\.(js|mjs|json|css|html)$/.test(name)) {
      const source = await fs.readFile(path.join(directory, name));
      if (source.length >= 1024 && !available.has(`${name}.gz`))
        throw new Error(`Missing gzip: ${name}`);
    }
  }
  if (!verifiedGzipFiles) throw new Error('No gzip resources verified');
  return { directory, verifiedGzipFiles, sourceBytes, gzipBytes };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const group = process.argv[2];
  if (!['js', 'rust', 'mixed'].includes(group)) throw new Error('Specify js, rust or mixed');
  console.log(JSON.stringify(await verifyCompression(fileURLToPath(new URL(`../dist/${group}/`, import.meta.url))), null, 2));
}
