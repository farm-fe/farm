import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

export function assertGlobDependencyClosure(metadata) {
  if (!Array.isArray(metadata?.packages) || !Array.isArray(metadata?.resolve?.nodes) ||
      !Array.isArray(metadata.workspace_members) || !metadata.workspace_members.length) {
    throw new Error('Complete Cargo metadata with a resolved workspace graph is required');
  }
  const forbidden = metadata.packages.filter(pkg => ['wax', 'tardar'].includes(pkg.name));
  if (forbidden.length) {
    throw new Error(`Old glob dependencies remain: ${forbidden.map(pkg => `${pkg.name}@${pkg.version}`).join(', ')}`);
  }
  const nodes = new Map(metadata.resolve.nodes.map(node => [node.id, node]));
  if (metadata.workspace_members.some(id => !nodes.has(id))) {
    throw new Error('Workspace member missing from the resolved graph');
  }
  const core = metadata.packages.find(pkg => pkg.name === 'farmfe_core');
  const glob = metadata.packages.find(pkg => pkg.name === 'farmfe_ecosystem_glob');
  if (!core || !glob || !nodes.get(core.id)?.deps?.some(dep => dep.pkg === glob.id)) {
    throw new Error('Core must depend directly on the new Farm glob crate');
  }
  return {packages: metadata.packages.length, nodes: nodes.size, core: core.version, glob: glob.version};
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const file = process.argv[2];
  if (!file) throw new Error('Pass actual Cargo metadata, not Cargo.lock or --no-deps output');
  console.log(JSON.stringify(assertGlobDependencyClosure(JSON.parse(fs.readFileSync(file, 'utf8').replace(/^\uFEFF/, '')))));
}
