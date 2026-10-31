import assert from 'node:assert/strict';
import crypto from 'node:crypto';

const hash = value => crypto.createHash('sha256').update(JSON.stringify(value)).digest('hex');
const sorted = values => [...new Set(values)].sort();
const roots = ['packages/core', 'packages/cli', 'packages/runtime', 'packages/runtime-plugin-hmr', 'packages/runtime-plugin-import-meta'];
const local = value => /^(workspace:|file:|link:)/.test(value);
// Deliberately narrow audited source boundaries. Manifests, config, runner and
// every other owner remain full execution until their implicit inputs are proved.
export const auditedSuiteLeaves = ['examples/react', 'js-plugins/babel'];

// This decision is separate from execution authority. A caller must verify a
// successful same-chain baseline before using the selected sets in a runner.
export function suiteImpact({ before, after, changes, inventory, auditedOwners = auditedSuiteLeaves }) {
  assert.ok(before && after && Array.isArray(changes));
  assert.equal(inventory.fileEnumerationComplete, true);
  const graph = new Map(), issues = [];
  for (const files of [before, after]) {
    const manifests = Object.entries(files).filter(([file]) => /^(packages|js-plugins|rust-plugins|examples)\/[^/]+\/package\.json$/.test(file));
    const names = new Map();
    for (const [file, text] of manifests) {
      const pkg = JSON.parse(text), directory = file.slice(0, -13);
      if (!pkg.name) { issues.push('missing-owner'); continue; }
      names.set(pkg.name, [...(names.get(pkg.name) ?? []), directory]);
      if (!graph.has(directory)) graph.set(directory, new Set());
    }
    for (const [file, text] of manifests) {
      const pkg = JSON.parse(text), directory = file.slice(0, -13);
      if (!graph.has(directory)) continue;
      for (const [name, version] of Object.entries({ ...pkg.dependencies, ...pkg.devDependencies, ...pkg.optionalDependencies, ...pkg.peerDependencies })) {
        const targets = names.get(name) ?? [];
        if (targets.length > 1) issues.push('ambiguous-owner');
        if (typeof version !== 'string') { issues.push('invalid-dependency'); continue; }
        if (version.startsWith('npm:')) { issues.push('unresolved-package-alias'); continue; }
        if (local(version) && (!version.startsWith('workspace:') || !/^(\*|\^|~|[~^=]?\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?)$/.test(version.slice(10)) || targets.length !== 1)) {
          issues.push('unresolved-local-dependency'); continue;
        }
        for (const target of targets) graph.get(directory).add(target);
      }
    }
  }
  // The PostCSS TS suite resolves UnoCSS through this Example's manifest.
  if (graph.has('js-plugins/postcss') && graph.has('examples/vite-adapter-vue')) graph.get('js-plugins/postcss').add('examples/vite-adapter-vue');
  const changed = sorted(changes.flatMap(row => [row.oldPath, row.path]).filter(Boolean));
  let full = issues.length > 0;
  const affected = new Set();
  for (const file of changed) {
    const directory = file.split('/').slice(0, 2).join('/');
    if (!graph.has(directory) || file.startsWith('rust-plugins/')) { full = true; continue; }
    affected.add(directory);
    if (!auditedOwners.includes(directory) || !file.startsWith(directory + '/src/') || !/\.(ts|tsx|js|jsx|css|scss|svg)$/.test(file)) {
      full = true; issues.push('unreviewed-product-boundary');
    }
  }
  for (let more = true; more;) {
    more = false;
    for (const [node, dependencies] of graph) if (!affected.has(node) && [...dependencies].some(dep => affected.has(dep))) {
      affected.add(node); more = true;
    }
  }
  if (roots.some(root => affected.has(root))) {
    for (const root of roots) if (!graph.has(root)) issues.push('required-shared-root-missing');
    full = true;
  }
  // The dynamic Examples recipe has shared prerequisites outside package.json.
  // Unknown mapping is full execution, never an omitted suite.
  if (inventory.fullOnlyReasons?.some(row => row.code.startsWith('unreviewed-')
    || row.code !== 'ambiguous-package-name' && (!row.file || affected.has(row.file.split('/').slice(0, 2).join('/'))))) full = true;
  const sets = {};
  for (const kind of ['ts', 'examples-build', 'examples-browser', 'create-farm']) {
    const items = inventory.items.filter(row => row.kind === kind);
    const ids = sorted(items.filter(row => full || kind === 'create-farm' || affected.has(row.directory ?? row.file?.split('/').slice(0, 2).join('/'))).map(row => row.id));
    sets[kind] = { mode: ids.length === items.length ? 'all' : ids.length ? 'selected' : 'not-applicable', ids };
  }
  const result = { schemaVersion: 1, kind: 'ci-suite-impact-decision', inventoryHash: inventory.inventoryHash,
    full, reasons: sorted(issues), changed, affectedOwners: sorted(affected), sets, executionAuthorized: false };
  return { ...result, decisionHash: hash(result) };
}
