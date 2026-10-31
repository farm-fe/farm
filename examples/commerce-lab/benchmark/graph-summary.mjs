import path from 'node:path';

export function summarizeGraph({ modules, edges }, { projectRoot, aliases = [] } = {}) {
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
  const application = modules.filter(module => projectRoot
    ? graphSourcePaths(module.id, projectRoot, aliases).some(file => /^(apps|packages)\//.test(relativePath(projectRoot, file) ?? ''))
    : /^(apps|packages)\//.test(module.id.replaceAll('\\', '/')));
  const dependencies = modules.filter(module => module.id.replaceAll('\\', '/').includes('/node_modules/'));
  const packages = new Set(dependencies.filter(module => module.packageName).map(module => `${module.packageName}@${module.packageVersion}`));
  return { retainedModules: modules.length, applicationModules: application.length,
    dependencyModules: dependencies.length, distinctDependencyPackages: packages.size,
    queryModules: modules.filter(module => module.id.includes('?')).length, edges: retainedPairs.size,
    tracedEdges: pairs.size, edgesWithUnretainedEndpoints: pairs.size - retainedPairs.size,
    limitation: 'The trace records retained post-optimization modules but can retain edges to pruned modules. Query modules are counted by identity, not asserted to be all plugin-generated modules.' };
}

const slash = value => value.replaceAll('\\', '/');
const pathApi = value => /^[A-Za-z]:\//.test(slash(value)) ? path.win32 : path.posix;
function absolutePath(value) {
  if (typeof value !== 'string') throw new Error('Non-string graph source path');
  const api = pathApi(value), normalized = slash(api.normalize(slash(value)));
  if (!api.isAbsolute(normalized)) throw new Error('Absolute graph root required');
  return normalized.replace(/\/$/, '');
}
function relativePath(root, file) {
  const api = pathApi(root), relative = slash(api.relative(root, file));
  return relative === '..' || relative.startsWith('../') || api.isAbsolute(relative) ? null : relative;
}

/** Resolve filesystem comparisons only; original graph/query/virtual IDs stay intact. */
export function graphSourcePaths(id, projectRoot, aliases = []) {
  if (typeof id !== 'string') throw new Error('Non-string actual graph identity');
  if (id.startsWith('\0') || /^(virtual:|farm-virtual:|~icons\/)/.test(id)) return [];
  const source = slash(id).split('?')[0].replace(/^\/@fs\//, '');
  if (!source || (/^[A-Za-z][A-Za-z0-9+.-]*:/.test(source) && !/^[A-Za-z]:\//.test(source))) return [];
  const project = absolutePath(projectRoot), api = pathApi(project);
  const mappings = aliases.map(row => ({ from: absolutePath(row.from), to: absolutePath(row.to) }))
    .sort((a, b) => b.from.length - a.from.length);
  const origins = [...new Set([project, ...mappings.filter(row => row.to === project).map(row => row.from)])];
  const results = new Set();
  for (const origin of origins) {
    let file = absolutePath(api.resolve(origin, source));
    for (const alias of mappings) {
      const relative = relativePath(alias.from, file);
      if (relative !== null) { file = absolutePath(pathApi(alias.to).resolve(alias.to, relative)); break; }
    }
    results.add(file);
  }
  return [...results];
}

/** Coverage of input identities, separate from retained module counts. */
export function generatedCoverage(expected, stages, generatedRoot, projectRoot, { aliases = [] } = {}) {
  const root = absolutePath(generatedRoot), project = absolutePath(projectRoot);
  const observed = new Set(), categories = {};
  let filesystemIdentities = 0, outsideGeneratedIdentities = 0;
  for (const [stage, values] of Object.entries(stages)) {
    const ids = [...new Set(values)];
    if (ids.some(id => typeof id !== 'string')) throw new Error('Non-string actual graph identity');
    categories[stage] = { identities: ids.length, query: ids.filter(id => id.includes('?')).length,
      virtual: ids.filter(id => id.startsWith('\0') || /^(virtual:|farm-virtual:|~icons\/)/.test(id)).length,
      css: ids.filter(id => /\.(css|scss|sass)(\?|$)/.test(id)).length,
      assets: ids.filter(id => /\.(svg|png|jpe?g|webp|wasm)(\?|$)/.test(id)).length };
    for (const id of ids) {
      const paths = graphSourcePaths(id, project, aliases);
      if (!paths.length) continue;
      filesystemIdentities++;
      const relatives = [...new Set(paths.map(file => relativePath(root, file)).filter(file => file !== null))];
      if (relatives.length === 1) observed.add(relatives[0]);
      else outsideGeneratedIdentities++;
    }
  }
  const files = [...new Set(expected.map(slash))];
  if (files.length !== expected.length || files.some(file => !file || file.startsWith('/') || /^[A-Za-z]:/.test(file) || file.split('/').includes('..') || file.includes('?'))) throw new Error('Invalid expected generated source list');
  const missing = files.filter(file => !observed.has(file)).sort();
  return { generatedFiles: files.length, generatedCovered: files.length - missing.length, missing, categories,
    reason: missing.length ? `${missing.length} generated source identities are absent after canonical-root path resolution` : null,
    pathNormalization: { projectRoot: project, generatedRoot: root, aliases, filesystemIdentities, outsideGeneratedIdentities } };
}
