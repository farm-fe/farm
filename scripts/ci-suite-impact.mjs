import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import { unifiedGraph } from './unified-impact-graph.mjs';
import { directoryImpactRules } from './ci-directory-impact-rules.mjs';
import { canonicalSuiteFile } from './ci-suite-inventory.mjs';

const hash = value => crypto.createHash('sha256').update(JSON.stringify(value)).digest('hex');
const sorted = values => [...new Set(values)].sort();

const matches = (file, rule) => rule.endsWith('/') ? file.startsWith(rule) : file === rule;

// This decision is separate from execution authority. A caller must verify a
// successful same-chain baseline before using the selected sets in a runner.
export function suiteImpact({ before, after, changes, inventory }) {
  assert.ok(before && after && Array.isArray(changes));
  assert.equal(inventory.fileEnumerationComplete, true);
  const canonical = unifiedGraph([before, after]);
  const issues = [...canonical.issues];
  const graph = new Map(Object.entries(canonical.nodes).map(([key, node]) => [key, new Set(node.testDeps)]));
  const changed = sorted(changes.flatMap(row => [row.oldPath, row.path]).filter(Boolean));
  let full = issues.length > 0;
  const affected = new Set(), explanations = [];
  // Only committed ordinary engineering Markdown is outside these suite inputs.
  // Missing metadata is not evidence; inspect both sides of deletions/renames and
  // mode changes. Docs quality and Unit remain independent CI gates; benchmarks run locally.
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
    const owners = Object.entries(canonical.nodes).filter(([, node]) => node.kind !== 'input' && file.startsWith(node.directory + '/'));
    const depth = Math.max(0, ...owners.map(([, node]) => node.directory.length));
    const direct = owners.filter(([, node]) => node.directory.length === depth).map(([key]) => key);
    const inputs = Object.entries(canonical.nodes).filter(([, node]) => node.kind === 'input' && node.paths.some(rule => matches(file, rule))).map(([key]) => key);
    if (!direct.length && !inputs.length) {
      full = true; issues.push('unmapped-directory-input');
      explanations.push({ path: file, owner: null, reason: 'unknown-full' });
      continue;
    }
    if (inputs.length) issues.push('global-directory-input');
    for (const owner of [...direct, ...inputs]) {
      affected.add(owner);
      explanations.push({ path: file, owner: canonical.nodes[owner].directory, reason: inputs.includes(owner) ? 'global' : 'directory-owner' });
    }
  }

  for (let more = true; more;) {
    more = false;
    for (const [node, dependencies] of graph) if (!affected.has(node) && [...dependencies].some(dep => affected.has(dep))) {
      affected.add(node); more = true;
      explanations.push({ owner: canonical.nodes[node].directory, reason: 'dependent', dependencies: sorted([...dependencies].filter(dep => affected.has(dep))) });
    }
  }
  const npmNodes = Object.entries(canonical.nodes).filter(([, node]) => node.kind === 'npm');
  full ||= npmNodes.length > 0 && npmNodes.every(([key]) => affected.has(key));
  const affectedDirectories = new Set([...affected].map(key => canonical.nodes[key].directory));
  if (inventory.fullOnlyReasons?.some(row => row.code.startsWith('unreviewed-')
    || row.code !== 'ambiguous-package-name' && (!row.file || affectedDirectories.has(row.file.split('/').slice(0, 2).join('/'))))) full = true;
  const sets = {};
  for (const kind of ['ts', 'examples-build', 'examples-browser', 'create-farm']) {
    const items = inventory.items.filter(row => row.kind === kind);
    const ids = sorted(items.filter(row => full || kind === 'create-farm' || affectedDirectories.has(row.directory ?? row.file?.split('/').slice(0, 2).join('/'))).map(row => row.id));
    sets[kind] = { mode: ids.length === items.length ? 'all' : ids.length ? 'selected' : 'not-applicable', ids };
  }
  const result = { schemaVersion: 1, kind: 'ci-suite-impact-decision', inventoryHash: inventory.inventoryHash,
    rulesVersion: directoryImpactRules.version, graphHash: canonical.graphHash, explanations,
    full, reasons: sorted(issues), changed, affectedOwners: sorted([...affectedDirectories].filter(directory => !directory.startsWith('@inputs/'))), sets, executionAuthorized: false };
  return { ...result, decisionHash: hash(result) };
}

export function npmDirectoryGraph(snapshots) {
  const canonical = unifiedGraph(snapshots);
  const graph = new Map();
  for (const [key, node] of Object.entries(canonical.nodes)) if (node.kind === 'npm') {
    graph.set(node.directory, new Set(node.preparationDeps.filter(dep => canonical.nodes[dep]?.kind === 'npm').map(dep => canonical.nodes[dep].directory)));
  }
  return { graph, issues: canonical.issues, graphHash: canonical.graphHash };
}
