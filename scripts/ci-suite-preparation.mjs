import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { npmDirectoryGraph } from './ci-suite-impact.mjs';
import { readGitSuiteFiles } from './ci-suite-inventory.mjs';
import { actualSuiteInventory, executionPlan } from './ci-suite-execution.mjs';

const bootstrap = ['packages/cli', 'packages/runtime', 'packages/runtime-plugin-hmr',
  'packages/runtime-plugin-import-meta', 'packages/core', 'packages/plugin-tools'];
export function preparationCommands(files, inventory, plan) {
  assert.ok(['ts', 'examples-build'].includes(plan.kind));
  assert.equal(plan.inventoryHash, inventory.inventoryHash);
  const items = plan.ids.map(id => {
    const item = inventory.items.find(row => row.id === id && row.kind === plan.kind);
    assert.ok(item, 'Unknown preparation suite'); return item;
  });
  if (plan.kind === 'ts' && items.length === 0) return [];
  const {graph, issues} = npmDirectoryGraph([files]);
  if (issues.length) {
    assert.equal(plan.mode, 'all', 'Unresolved dependencies require full preparation');
    const plugins = [...graph.keys()].filter(directory => directory.startsWith('js-plugins/')).sort();
    if (plugins.includes('js-plugins/dts')) plugins.splice(plugins.indexOf('js-plugins/dts'), 1);
    if (graph.has('js-plugins/dts')) plugins.unshift('js-plugins/dts');
    return [...bootstrap, ...plugins].map(directory => ({directory,
      args: ['run', directory === 'packages/core' ? 'build:cjs' : 'build'], reason: 'unresolved-graph-full-recipe'}));
  }
  const selected = new Set();
  function include(directory) {
    if (selected.has(directory)) return;
    assert.ok(graph.has(directory), 'Missing preparation owner');
    selected.add(directory);
    for (const dependency of graph.get(directory)) include(dependency);
  }
  for (const item of items) include(item.directory ?? item.file.split('/').slice(0, 2).join('/'));
  // Examples also prepares the shared SDK for the downstream eight-template
  // create-farm gate, including an explicitly empty Examples selection.
  const ordered = [], pending = new Set(), done = new Set();
  const exampleTasks = new Set(items.filter(row => row.kind === 'examples-build').map(row => row.directory));
  const docs = ['packages/docs', 'packages/docs-theme-default'];
  function visit(directory) {
    if (bootstrap.includes(directory) || directory.startsWith('rust-plugins/') || exampleTasks.has(directory)) return;
    const cohort = docs.includes(directory) ? docs.filter(row => graph.has(row)) : [directory];
    const key = cohort[0];
    if (done.has(key)) return;
    assert.ok(!pending.has(key), 'Cyclic JS preparation dependencies');
    pending.add(key);
    for (const member of cohort) {
      for (const dependency of [...graph.get(member)].sort()) if (!cohort.includes(dependency)) visit(dependency);
    }
    pending.delete(key); done.add(key);
    for (const member of cohort) if (JSON.parse(files[member + '/package.json']).scripts?.build) ordered.push(member);
  }
  for (const directory of [...selected].sort()) visit(directory);
  return [...bootstrap, ...ordered].map(directory => ({directory,
    args: ['run', directory === 'packages/core' ? 'build:cjs' : 'build']}));
}
export function executePreparation(commands, files, root, execute = (command, args, options) => execFileSync(command, args, options)) {
  for (const row of commands) {
    const manifest = row.directory + '/package.json';
    assert.equal(fs.readFileSync(path.join(root, manifest), 'utf8').replaceAll('\r\n', '\n'), files[manifest].replaceAll('\r\n', '\n'), 'Preparation manifest changed');
    const pkg = JSON.parse(files[manifest]);
    const script = pkg.scripts?.[row.args[1]];
    assert.ok(script && !/\bcargo\b|\bnapi\b|farm-plugin-tools build/.test(script), 'Native work is not JS preparation');
    console.log('[suite preparation] ' + row.directory + ' ' + row.args.join(' '));
    execute('pnpm', row.args, {cwd: path.join(root, row.directory), stdio: 'inherit', timeout: 300000});
  }
}
export function prepareSuite(inventory, plan, root = fileURLToPath(new URL('../', import.meta.url))) {
  const snapshot = readGitSuiteFiles(root);
  assert.deepEqual(snapshot.sourceIdentity, inventory.sourceIdentity, 'Preparation source changed');
  const commands = preparationCommands(snapshot.files, inventory, plan);
  executePreparation(commands, snapshot.files, root);
  return commands;
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const inventory = actualSuiteInventory();
  prepareSuite(inventory, executionPlan(inventory, 'ts'));
}
