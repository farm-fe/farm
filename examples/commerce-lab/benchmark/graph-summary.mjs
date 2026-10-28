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

/** Coverage of input identities, separate from either bundler's retained module count. */
export function generatedCoverage(expected, stages, generatedRoot, projectRoot) {
  const normalize = value => value.replaceAll('\\', '/').split('?')[0].replace(/^\/@fs\//, '');
  const absolute = normalize(generatedRoot).replace(/\/$/, '') + '/';
  const project = normalize(projectRoot).replace(/\/$/, '') + '/';
  const relative = absolute.startsWith(project) ? absolute.slice(project.length) : absolute;
  const observed = new Set();
  const categories = {};
  for (const [stage, values] of Object.entries(stages)) {
    const ids = [...new Set(values)];
    if (ids.some(id => typeof id !== 'string')) throw new Error('Non-string actual graph identity');
    categories[stage] = { identities: ids.length, query: ids.filter(id => id.includes('?')).length,
      virtual: ids.filter(id => id.startsWith('\0') || /^(virtual:|farm-virtual:|~icons\/)/.test(id)).length,
      css: ids.filter(id => /\.(css|scss|sass)(\?|$)/.test(id)).length,
      assets: ids.filter(id => /\.(svg|png|jpe?g|webp|wasm)(\?|$)/.test(id)).length };
    for (const id of ids) {
      const file = normalize(id);
      if (file.startsWith(absolute)) observed.add(file.slice(absolute.length));
      else if (file.startsWith(relative)) observed.add(file.slice(relative.length));
    }
  }
  const files = [...new Set(expected.map(normalize))];
  if (files.length !== expected.length || files.some(file => file.startsWith('/') || file.split('/').includes('..'))) throw new Error('Invalid expected generated source list');
  const missing = files.filter(file => !observed.has(file)).sort();
  return { generatedFiles: files.length, generatedCovered: files.length - missing.length, missing, categories };
}
