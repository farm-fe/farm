import assert from 'node:assert/strict';
import test from 'node:test';
import { summarizeGraph } from '../benchmark/graph-summary.mjs';

test('graph counts use retained module identities and edges rather than fixture rows', () => {
  const modules = [
    { id: 'apps/admin/main.tsx', packageName: '', packageVersion: '' },
    { id: 'apps/admin/style.scss?farm_css', packageName: '', packageVersion: '' },
    { id: '../../node_modules/react/index.js', packageName: 'react', packageVersion: '18.3.1' },
    { id: '../../node_modules/react/jsx-runtime.js', packageName: 'react', packageVersion: '18.3.1' },
  ];
  const result = summarizeGraph({ modules, edges: { 'apps/admin/main.tsx': [modules[1].id, modules[2].id, modules[2].id] } });
  assert.equal(result.retainedModules, 4);
  assert.equal(result.applicationModules, 2);
  assert.equal(result.dependencyModules, 2);
  assert.equal(result.distinctDependencyPackages, 1);
  assert.equal(result.edges, 2);
  assert.equal(result.queryModules, 1);
  assert.throws(() => summarizeGraph({ modules: [...modules, modules[0]], edges: {} }), /duplicate/i);
  const pruned = summarizeGraph({ modules, edges: { missing: [modules[0].id, modules[0].id] } });
  assert.equal(pruned.edges, 0);
  assert.equal(pruned.tracedEdges, 1);
  assert.equal(pruned.edgesWithUnretainedEndpoints, 1);
});
