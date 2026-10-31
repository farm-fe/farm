import assert from 'node:assert/strict';
import {test} from 'node:test';
import {assertGlobDependencyClosure} from './assert-glob-dependency-closure.mjs';

function graph() {
  return {
    packages: [{id:'core', name:'farmfe_core', version:'2.2.0'}, {id:'glob', name:'farmfe_ecosystem_glob', version:'0.1.0'}],
    workspace_members: ['core','glob'],
    resolve: {nodes: [{id:'core', deps:[{name:'farmfe_ecosystem_glob',pkg:'glob'}]}, {id:'glob',deps:[]}]}
  };
}

test('requires a resolved graph and the actual Core to Farm glob edge', () => {
  assert.deepEqual(assertGlobDependencyClosure(graph()), {packages:2,nodes:2,core:'2.2.0',glob:'0.1.0'});
  assert.throws(() => assertGlobDependencyClosure({packages:[]}), /resolved workspace graph/);
  const missing = graph();
  missing.resolve.nodes[0].deps = [];
  assert.throws(() => assertGlobDependencyClosure(missing), /directly/);
  const partial = graph();
  partial.resolve.nodes.pop();
  assert.throws(() => assertGlobDependencyClosure(partial), /member missing/);
});

test('rejects old dependencies even when another workspace member introduces them', () => {
  for (const name of ['wax','tardar']) {
    const metadata = graph();
    metadata.packages.push({id:name,name,version:'0.1.0'});
    metadata.resolve.nodes.push({id:name,deps:[]});
    assert.throws(() => assertGlobDependencyClosure(metadata), /Old glob dependencies remain/);
  }
});
