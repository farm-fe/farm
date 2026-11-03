import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import YAML from 'yaml';

// A job's explicit map replaces workflow defaults. Omitted scopes in an
// explicit map are none. Only incoming reusable-call permissions are a ceiling.
const rank = { none: 0, read: 1, write: 2 };
function level(permissions, scope) {
  if (permissions === 'write-all') return 2;
  if (permissions === 'read-all') return 1;
  return rank[permissions?.[scope] ?? 'none'];
}
function requireSubset(requested, available, location) {
  const scopes = new Set(['actions', 'contents', 'checks', 'pull-requests', 'id-token',
    ...Object.keys(typeof requested === 'object' ? requested : {}),
    ...Object.keys(typeof available === 'object' ? available : {})]);
  for (const scope of scopes) {
    assert.ok(level(requested, scope) <= level(available, scope),
      `${location}: ${scope} permission exceeds caller (${level(requested, scope)} > ${level(available, scope)})`);
  }
}
function checkWorkflows(workflows) {
  let edges = 0;
  function visit(name, incoming, stack = []) {
    assert.ok(!stack.includes(name), `Reusable workflow cycle: ${[...stack, name].join(' -> ')}`);
    const workflow = workflows[name];
    assert.ok(workflow, `Missing reusable workflow: ${name}`);
    if (incoming && workflow.permissions !== undefined) requireSubset(workflow.permissions, incoming, name);
    const defaults = workflow.permissions ?? incoming ?? 'write-all';
    for (const [id, job] of Object.entries(workflow.jobs ?? {})) {
      const available = job.permissions ?? defaults;
      if (incoming) requireSubset(available, incoming, `${name}/${id}`);
      if (!job.uses?.startsWith('./.github/workflows/')) continue;
      const target = job.uses.slice('./.github/workflows/'.length);
      assert.ok(workflows[target]?.on && 'workflow_call' in workflows[target].on,
        `${name}/${id}: ${target} must declare workflow_call`);
      edges++;
      visit(target, available, [...stack, name]);
    }
  }
  for (const name of Object.keys(workflows)) visit(name);
  return edges;
}
const root = new URL('../.github/workflows/', import.meta.url);
const actual = Object.fromEntries(fs.readdirSync(root).filter(name => /\.ya?ml$/.test(name))
  .map(name => [name, YAML.parse(fs.readFileSync(new URL(name, root), 'utf8'))]));
function fixture(caller, callee) {
  return { 'a.yaml': { permissions: { contents: 'read' }, jobs: { call: { uses: './.github/workflows/b.yaml', ...caller } } },
    'b.yaml': { on: { workflow_call: {} }, permissions: { contents: 'read', actions: 'read' }, jobs: {}, ...callee } };
}
test('all local reusable workflow edges preserve the caller permission ceiling', () => {
  assert.ok(checkWorkflows(actual) > 10);
});
test('original additional-native caller without actions read is rejected', () => {
  const broken = structuredClone(actual);
  delete broken['ci.yaml'].jobs['call-rust-build-release'].permissions;
  assert.throws(() => checkWorkflows(broken), /rust-build-release.yaml: actions permission exceeds caller/);
});
test('explicit caller read permission permits artifact-reading callee', () => {
  assert.doesNotThrow(() => checkWorkflows(fixture({ permissions: { contents: 'read', actions: 'read' } })));
});
test('unspecified scopes in explicit caller map are none', () => {
  assert.throws(() => checkWorkflows(fixture({})), /actions permission exceeds caller/);
});
test('read does not authorize a nested write request', () => {
  assert.throws(() => checkWorkflows(fixture({ permissions: { contents: 'read', actions: 'read' } },
    { jobs: { upload: { permissions: { actions: 'write' } } } })), /actions permission exceeds caller/);
});
test('callee without explicit permissions inherits caller rather than repository default', () => {
  const w = fixture({});
  delete w['b.yaml'].permissions;
  w['b.yaml'].jobs = { nested: { uses: './.github/workflows/c.yaml' } };
  w['c.yaml'] = { on: { workflow_call: {} }, permissions: { actions: 'read' }, jobs: {} };
  assert.throws(() => checkWorkflows(w), /actions permission exceeds caller/);
});
test('workflow job overrides are not incorrectly limited by top-level defaults', () => {
  const w = fixture({ permissions: { contents: 'read', actions: 'read' } });
  assert.doesNotThrow(() => checkWorkflows(w));
});
test('call cycles fail instead of silently omitting a permission edge', () => {
  const w = fixture({ permissions: { contents: 'read', actions: 'read' } });
  w['a.yaml'].on = { workflow_call: {} };
  w['b.yaml'].jobs = { cycle: { uses: './.github/workflows/a.yaml' } };
  assert.throws(() => checkWorkflows(w), /Reusable workflow cycle/);
});
