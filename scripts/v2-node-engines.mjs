// Generic installed-graph audit adapted from bc577f8d; no historical consumer receipt is required.
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import semver from 'semver';

export function evaluateNodeEngines(packages, candidates) {
  return candidates.map(node => {
    const incompatible = packages.filter(item => item.engine && !semver.satisfies(node, item.engine));
    const offenders = incompatible.filter(item => !item.optional);
    return { node, satisfiesInstalledRanges: offenders.length === 0, offenders, optionalOffenders: incompatible.filter(item => item.optional) };
  });
}

export function collectInstalledDependencies(consumer, roots) {
  const manifestPath = path.join(consumer, 'package.json');
  const root = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  const visited = new Map();
  const packages = new Map();
  const missing = new Map();
  function visit(parent, name, optional) {
    const lookup = createRequire(parent).resolve.paths(name) ?? [];
    const filename = lookup.map(directory => path.join(directory, name, 'package.json')).find(candidate => fs.existsSync(candidate));
    if (!filename) {
      const key = parent + '\0' + name;
      const previous = missing.get(key);
      missing.set(key, { parent, name, optional: optional && (previous?.optional ?? true) });
      return;
    }
    const canonical = fs.realpathSync(filename);
    if (visited.has(canonical) && (!visited.get(canonical) || optional)) return;
    // Revisit an optional subtree when a required path also reaches it.
    visited.set(canonical, optional);
    const manifest = JSON.parse(fs.readFileSync(canonical, 'utf8'));
    packages.set(canonical, { name: manifest.name, version: manifest.version, engine: manifest.engines?.node, path: canonical, optional });
    const optionalNames = manifest.optionalDependencies ?? {};
    for (const dependency of Object.keys({ ...manifest.dependencies, ...optionalNames })) {
      visit(canonical, dependency, optional || Object.hasOwn(optionalNames, dependency));
    }
  }
  packages.set(manifestPath, { name: root.name, version: root.version, engine: root.engines?.node, path: manifestPath, optional: false });
  const optionalNames = root.optionalDependencies ?? {};
  for (const name of roots ?? Object.keys({ ...root.dependencies, ...optionalNames })) {
    visit(manifestPath, name, Object.hasOwn(optionalNames, name));
  }
  return { packages: [...packages.values()], missing: [...missing.values()] };
}
