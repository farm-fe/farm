import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const cli = require.resolve('vue-tsc/bin/vue-tsc.js');
const variant = process.env.COMMERCE_VUE_VARIANT ?? 'sfc';
assert.ok(['sfc', 'jsx'].includes(variant));
const file = variant === 'jsx' ? 'apps/vue-jsx-lab/Inventory.tsx' : 'apps/vue-lab/Inventory.vue';
const original = await fs.readFile(file, 'utf8');
const anchor = variant === 'jsx' ? 'stock={product.stock}' : ':stock="product.stock"';
assert.equal(original.split(anchor).length, 2);
const report = { kind: 'vue-types', variant, success: false, phases: [] };
function check(name, expectedSuccess) {
  const result = spawnSync(process.execPath, [cli, '--noEmit', '-p', variant === 'jsx' ? 'tsconfig.vue-jsx.json' : 'tsconfig.vue.json'], { encoding: 'utf8' });
  const output = `${result.stdout ?? ''}${result.stderr ?? ''}`;
  report.phases.push({ name, exitCode: result.status, output });
  assert.equal(result.error, undefined);
  assert.equal(result.status === 0, expectedSuccess, output);
  if (!expectedSuccess) assert.match(output, /Inventory\.(vue|tsx).*TS2322.*string.*number/);
}
try {
  check('baseline', true);
  await fs.writeFile(file, original.replace(anchor, variant === 'jsx' ? 'stock={String(product.stock)}' : ':stock="String(product.stock)"'));
  check('invalid-component-prop', false);
  await fs.writeFile(file, original);
  check('restored', true);
  report.success = true;
} finally {
  await fs.writeFile(file, original);
  await fs.mkdir('reports', { recursive: true });
  await fs.writeFile(`reports/vue-types-${variant}-${Date.now()}.json`, JSON.stringify(report, null, 2));
  console.log(JSON.stringify(report));
}
