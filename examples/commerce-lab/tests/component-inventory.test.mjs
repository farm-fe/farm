import assert from 'node:assert/strict';
import test from 'node:test';
import { analyzeComponentSources } from '../benchmark/component-inventory.mjs';
test('component inventory follows imports and aliases without counting unused or generated declarations', () => {
  const result = analyzeComponentSources({
    'apps/admin/main.tsx': 'import { Card as Tile } from "../../packages/ui"; const page=()=>import("./pages/Orders"); function Workspace(){return <Tile/>;} const app=<Workspace/>;',
    'packages/ui/index.tsx': 'export const Card=()=> <section/>; export function Unused(){return <aside/>;}',
    'apps/admin/pages/Orders.tsx': 'export default function Orders(){return <table/>;}',
    'packages/ui/unreachable.tsx': 'export function Unreachable(){return <div/>;}',
    'packages/extensions/baseline/Form.tsx': 'export function Generated(){return <form/>;}',
  }, ['apps/admin/main.tsx']);
  assert.equal(result.manualBusinessSharedCount, 1);
  assert.equal(result.pageCount, 1);
  assert.equal(result.infrastructureCount, 1);
  assert.deepEqual(result.components.filter(component => component.referenced).map(component => component.name).sort(), ['Card', 'Orders', 'Workspace']);
  assert.equal(result.components.find(component => component.name === 'Unused').referenced, false);
  assert.ok(!result.components.some(component => component.name === 'Generated' || component.name === 'Unreachable'));
});
