import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export function prepareRustPackageLicenses(root, plan) {
  const original = fs.readFileSync(path.join(root, 'LICENSE'));
  const added = [];
  const prepared = [];
  for (const pkg of plan.cargo) {
    // Existing published macro bytes are immutable; its explicit reuse evidence
    // is independent from preparation of new candidate archives.
    if (pkg.name === 'farmfe_macro_cache_item') continue;
    const manifest = path.join(root, pkg.manifestPath);
    const directory = path.dirname(manifest);
    if (fs.existsSync(path.join(directory, 'LICENSE'))) continue;
    assert.match(fs.readFileSync(manifest, 'utf8'), /^license\s*=\s*"MIT"\s*$/m, `Unreviewed Rust license: ${pkg.name}`);
    assert.ok(!fs.readdirSync(directory).some(name => /^(license|copying)/i.test(name)), `Existing alternative legal material requires review: ${pkg.name}`);
    prepared.push({ name: pkg.name, directory });
  }
  // Validate the entire missing-file set before the first source write.
  for (const pkg of prepared) {
    fs.writeFileSync(path.join(pkg.directory, 'LICENSE'), original, { flag: 'wx' });
    added.push(pkg.name);
  }
  return added;
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const root = fileURLToPath(new URL('../', import.meta.url));
  const plan = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
  console.log(JSON.stringify({ added: prepareRustPackageLicenses(root, plan) }));
}
