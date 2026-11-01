import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import path from 'node:path';
import { directoryImpactRules } from './ci-directory-impact-rules.mjs';
import { canonicalSuiteFile } from './ci-suite-inventory.mjs';

const hash = value => crypto.createHash('sha256').update(JSON.stringify(value)).digest('hex');
const sorted = values => [...new Set(values)].sort();
const roots = ['packages/core', 'packages/cli', 'packages/runtime', 'packages/runtime-plugin-hmr', 'packages/runtime-plugin-import-meta'];

const matches = (file, rule) => rule.endsWith('/') ? file.startsWith(rule) : file === rule;

// This decision is separate from execution authority. A caller must verify a
// successful same-chain baseline before using the selected sets in a runner.
export function suiteImpact({ before, after, changes, inventory }) {
  assert.ok(before && after && Array.isArray(changes));
  assert.equal(inventory.fileEnumerationComplete, true);
  const { graph, issues } = npmDirectoryGraph([before, after]);
  for (const rule of directoryImpactRules.dependencies) {
    for (const [consumer, dependencies] of graph) {
      if (matches(consumer + '/', rule.consumer.endsWith('/') ? rule.consumer : rule.consumer + '/')) {
        for (const dependency of rule.dependencies) {
          if (graph.has(dependency)) dependencies.add(dependency);
          else issues.push('declared-directory-dependency-missing');
        }
      }
    }
  }
  const changed = sorted(changes.flatMap(row => [row.oldPath, row.path]).filter(Boolean));
  let full = issues.length > 0;
  const affected = new Set(), explanations = [];
  // Only committed ordinary engineering Markdown is outside these suite inputs.
  // Missing metadata is not evidence; inspect both sides of deletions/renames and
  // mode changes. Docs quality, Unit and performance remain independent gates.
  const documentationOnly = file => {
    canonicalSuiteFile(file);
    if (!file.startsWith('docs/experiences/') || !file.endsWith('.md')) return false;
    const rows = changes.filter(row => row.path === file || row.oldPath === file);
    return rows.length > 0 && rows.every(row => {
      if (!Object.hasOwn(row, 'beforeEntry') || !Object.hasOwn(row, 'afterEntry')) return false;
      const entries = [row.beforeEntry, row.afterEntry].filter(Boolean);
      return entries.length > 0 && entries.every(entry => entry.file === file && entry.type === 'blob' && entry.mode === '100644');
    });
  };
  for (const file of changed) {
    if (documentationOnly(file)) continue;
    const directory = file.split('/').slice(0, 2).join('/');
    const global = directoryImpactRules.global.find(rule => matches(file, rule));
    if (global) {
      full = true; issues.push('global-directory-input');
      explanations.push({ path: file, owner: graph.has(directory) ? directory : null, reason: 'global', rule: global });
      if (graph.has(directory)) affected.add(directory);
      continue;
    }
    if (!graph.has(directory)) {
      full = true; issues.push('unmapped-directory-input');
      explanations.push({ path: file, owner: null, reason: 'unknown-full' });
      continue;
    }
    affected.add(directory);
    explanations.push({ path: file, owner: directory, reason: 'directory-owner' });
  }

  for (let more = true; more;) {
    more = false;
    for (const [node, dependencies] of graph) if (!affected.has(node) && [...dependencies].some(dep => affected.has(dep))) {
      affected.add(node); more = true;
      explanations.push({ owner: node, reason: 'dependent', dependencies: sorted([...dependencies].filter(dep => affected.has(dep))) });
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
    rulesVersion: directoryImpactRules.version, explanations,
    full, reasons: sorted(issues), changed, affectedOwners: sorted(affected), sets, executionAuthorized: false };
  return { ...result, decisionHash: hash(result) };
}

export function npmDirectoryGraph(snapshots) {
  const graph = new Map(), issues = [];
  for (const files of snapshots) {
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
        if (typeof version !== 'string') { issues.push('invalid-dependency'); continue; }
        let targets = names.get(name) ?? [];
        const localPath = /^(?:file:|link:)(.*)$/.exec(version) ?? /^workspace:(\..*)$/.exec(version);
        if (localPath) {
          const target = path.posix.normalize(path.posix.join(directory, localPath[1]));
          targets = graph.has(target) ? [target] : [];
          if (!targets.length) issues.push('unresolved-local-dependency');
        } else if (version.startsWith('workspace:') || version.startsWith('npm:')) {
          const spec = version.slice(version.indexOf(':') + 1);
          const alias = /^((?:@[^/]+\/)?[^@]+)@([^@]+)$/.exec(spec);
          if (alias) targets = names.get(alias[1]) ?? [];
          else if (!/^(\*|\^|~|[~^=]?\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?)$/.test(spec)) targets = [];
          if (targets.length !== 1) issues.push(version.startsWith('npm:') ? 'unresolved-package-alias' : 'unresolved-local-dependency');
        }
        if (targets.length > 1) issues.push('ambiguous-owner');
        for (const target of targets) graph.get(directory).add(target);
      }
    }
  }
  return { graph, issues };
}
