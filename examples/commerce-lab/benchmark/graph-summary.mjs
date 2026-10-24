export function summarizeGraph({ modules, edges }) {
  if (!Array.isArray(modules) || !edges || typeof edges !== 'object') throw new Error('Invalid traced graph');
  const ids = new Set(modules.map(module => module.id));
  if (ids.size !== modules.length) throw new Error('Duplicate module identity');
  const pairs = new Set();
  const retainedPairs = new Set();
  for (const [source, targets] of Object.entries(edges)) {
    for (const target of targets) {
      pairs.add(JSON.stringify([source, target]));
      if (ids.has(source) && ids.has(target)) retainedPairs.add(JSON.stringify([source, target]));
    }
  }
  const application = modules.filter(module => /^(apps|packages)\//.test(module.id.replaceAll('\\', '/')));
  const dependencies = modules.filter(module => module.id.replaceAll('\\', '/').includes('/node_modules/'));
  const packages = new Set(dependencies.filter(module => module.packageName).map(module => `${module.packageName}@${module.packageVersion}`));
  return { retainedModules: modules.length, applicationModules: application.length,
    dependencyModules: dependencies.length, distinctDependencyPackages: packages.size,
    queryModules: modules.filter(module => module.id.includes('?')).length, edges: retainedPairs.size,
    tracedEdges: pairs.size, edgesWithUnretainedEndpoints: pairs.size - retainedPairs.size,
    limitation: 'The trace records retained post-optimization modules but can retain edges to pruned modules. Query modules are counted by identity, not asserted to be all plugin-generated modules.' };
}
