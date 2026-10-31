import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { candidateJsBuilds, orderJsCandidates } from './prepare-v2-candidate-js.mjs';
test('candidate producer builds public JS in dependency order and never recompiles native wrappers', () => {
  const plan = JSON.parse(fs.readFileSync(new URL('../docs/releases/v2-candidate.json', import.meta.url)));
  const recipe = candidateJsBuilds(plan);
  assert.equal(new Set(recipe).size, recipe.length);
  assert.ok(recipe.indexOf('packages/core') < recipe.indexOf('js-plugins/babel'));
  assert.ok(recipe.indexOf('packages/core') < recipe.indexOf('js-plugins/dts'));
  assert.ok(recipe.indexOf('js-plugins/dts') < recipe.indexOf('js-plugins/babel'));
  assert.ok(recipe.indexOf('js-plugins/babel') < recipe.indexOf('js-plugins/react-compiler'));
  assert.ok(!recipe.some(dir => dir.startsWith('rust-plugins/') || dir === 'packages/create-farm'));
  for (const directory of recipe) {
    const manifest = JSON.parse(fs.readFileSync(new URL(`../${directory}/package.json`, import.meta.url)));
    assert.doesNotMatch(manifest.scripts.build, /\bcargo\b|\bnapi\b|farm-plugin-tools build/);
    for (const file of fs.readdirSync(new URL(`../${directory}/`, import.meta.url)).filter(name => /^farm.*\.config\.(ts|mjs|js)$/.test(name))) {
      const source = fs.readFileSync(new URL(`../${directory}/${file}`, import.meta.url), 'utf8');
      if (/from ['"]@farmfe\/js-plugin-dts['"]/.test(source)) assert.ok(recipe.indexOf('js-plugins/dts') < recipe.indexOf(directory), `${directory}/${file} needs the complete DTS build first`);
    }
  }
  assert.ok(recipe.indexOf('js-plugins/less') < recipe.indexOf('js-plugins/visualizer'));
  assert.ok(recipe.indexOf('js-plugins/postcss') < recipe.indexOf('js-plugins/visualizer'));
  const dts = JSON.parse(fs.readFileSync(new URL('../js-plugins/dts/package.json', import.meta.url)));
  assert.equal(dts.scripts.build, 'farm build && npm run build:tsc', 'Both CJS and ESM/types bootstrap stages are required');
  assert.equal(dts.exports['.'].import, './dist/index.js');
});

test('unsorted clean JS manifests include dev-dependency prerequisites without consulting existing dist', () => {
  const item = (name, dependencies = {}, devDependencies = {}) => ({ directory: `js-plugins/${name}`, manifest: { name: `@farmfe/js-plugin-${name}`, dependencies, devDependencies } });
  const dts = item('dts'), babel = item('babel', {}, { '@farmfe/js-plugin-dts': 'workspace:*' });
  const react = item('react-compiler', { '@farmfe/js-plugin-babel': 'workspace:^' }, { '@farmfe/js-plugin-dts': 'workspace:*' });
  assert.deepEqual(orderJsCandidates([react, babel, dts]), ['js-plugins/dts', 'js-plugins/babel', 'js-plugins/react-compiler']);
  assert.throws(() => orderJsCandidates([babel]), /Missing selected JS build prerequisite/);
  assert.throws(() => orderJsCandidates([item('a', {}, { '@farmfe/js-plugin-b': 'workspace:*' }), item('b', { '@farmfe/js-plugin-a': 'workspace:*' })]), /Cyclic JS build prerequisite/);
});
