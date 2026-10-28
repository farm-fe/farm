import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

// Mapping follows Cargo's documented registry publish API. Inputs are actual
// Cargo metadata from the independently tested normalized registry packages.
// https://doc.rust-lang.org/cargo/reference/registry-web-api.html#publish
export function uploadMetadata(pkg) {
  assert.match(pkg.source ?? '', /^registry\+/); assert.ok(pkg.description?.trim(), `Missing Rust package description: ${pkg.name}`);
  const deps = pkg.dependencies.map(dep => {
    assert.ok(!dep.path && (!dep.source || dep.source.startsWith('registry+')), `Non-registry publish dependency: ${dep.name}`);
    assert.ok(!dep.registry || ['https://github.com/rust-lang/crates.io-index', 'sparse+https://index.crates.io/'].includes(dep.registry), 'Unreviewed alternate Rust registry');
    return { name: dep.name, version_req: dep.req, features: dep.features, optional: dep.optional, default_features: dep.uses_default_features,
      target: dep.target, kind: dep.kind ?? 'normal', registry: null, explicit_name_in_toml: dep.rename ?? null };
  });
  let readme = null;
  if (pkg.readme) {
    const root = fs.realpathSync(path.dirname(pkg.manifest_path));
    const file = fs.realpathSync(path.resolve(root, pkg.readme));
    assert.ok(file.startsWith(root + path.sep), 'Rust README escapes package');
    assert.ok(fs.statSync(file).size <= 1024 * 1024, 'Rust README exceeds bound');
    readme = fs.readFileSync(file, 'utf8');
  }
  assert.ok(pkg.license || pkg.license_file, 'Missing Rust license declaration');
  return { name: pkg.name, vers: pkg.version, deps, features: pkg.features, authors: pkg.authors, description: pkg.description,
    documentation: pkg.documentation, homepage: pkg.homepage, readme, readme_file: pkg.readme,
    keywords: pkg.keywords, categories: pkg.categories, license: pkg.license, license_file: pkg.license_file,
    repository: pkg.repository, badges: {}, links: pkg.links, rust_version: pkg.rust_version };
}
export function rustPublicationOrder(plan, metadata, { includeUploadMetadata = true } = {}) {
  const selected = new Map(plan.cargo.map(record => {
    const matches = metadata.packages.filter(pkg => pkg.name === record.name && pkg.version === record.version);
    assert.equal(matches.length, 1, `Missing exact Rust SDK package: ${record.name}`);
    return [record.name, matches[0]];
  }));
  const order = [], visited = new Set(), visiting = new Set();
  function visit(name) {
    if (visited.has(name)) return;
    assert.ok(!visiting.has(name), 'Rust publication dependency cycle'); visiting.add(name);
    const pkg = selected.get(name);
    const dependencies = [...new Set(pkg.dependencies.filter(dep => dep.kind !== 'dev' && selected.has(dep.name)).map(dep => dep.name))].sort();
    for (const dependency of dependencies) visit(dependency);
    visiting.delete(name); visited.add(name); order.push({ name, version: pkg.version, dependencies, ...(includeUploadMetadata ? { metadata: uploadMetadata(pkg) } : {}) });
  }
  for (const name of [...selected.keys()].sort()) visit(name);
  return order;
}
