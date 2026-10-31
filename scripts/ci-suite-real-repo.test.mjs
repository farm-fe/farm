import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { discoverSuiteInventory, suitePolicy } from './ci-suite-inventory.mjs';
import { suiteImpact } from './ci-suite-impact.mjs';

test('real repository impact sets equal independent Examples and JS expectations', () => {
  const root = fileURLToPath(new URL('../', import.meta.url)), files = {};
  for (const file of ['package.json', ...Object.keys(suitePolicy.inputs)]) files[file] = fs.readFileSync(path.join(root, file), 'utf8');
  function walk(folder) {
    for (const entry of fs.readdirSync(path.join(root, folder), { withFileTypes: true })) {
      if (entry.isSymbolicLink() || ['node_modules', '.git', 'dist', 'target'].includes(entry.name)) continue;
      const file = folder + '/' + entry.name;
      if (entry.isDirectory()) walk(file);
      else if (file.endsWith('.spec.ts') || /^(packages|js-plugins|rust-plugins|examples)\/[^/]+\/package\.json$/.test(file)
        || /^examples\/[^/]+\/(index\.html|e2e\.spec\.mjs)$/.test(file)) files[file] = fs.readFileSync(path.join(root, file), 'utf8');
    }
  }
  for (const scope of ['packages', 'js-plugins', 'rust-plugins', 'examples']) walk(scope);
  const directories = fs.readdirSync(path.join(root, 'examples'), { withFileTypes: true }).filter(row => row.isDirectory()).map(row => row.name);
  const inventory = discoverSuiteInventory(files, suitePolicy, null, directories);
  const plan = file => suiteImpact({ before: files, after: files, changes: [{ path: file }], inventory });
  const react = plan('examples/react/src/main.tsx');
  assert.equal(react.full, false); assert.deepEqual(react.sets.ts.ids, []);
  assert.deepEqual(react.sets['examples-build'].ids, ['examples-build:examples/react']);
  assert.deepEqual(react.sets['examples-browser'].ids, ['examples-browser:examples/react']);
  const babel = plan('js-plugins/babel/src/index.ts');
  assert.equal(babel.full, false); assert.deepEqual(babel.affectedOwners, ['js-plugins/babel', 'js-plugins/react-compiler']);
  assert.deepEqual(babel.sets.ts.ids, ['ts:js-plugins/react-compiler/tests/compiler.spec.ts']);
  assert.deepEqual(babel.sets['examples-build'].ids, []); assert.deepEqual(babel.sets['examples-browser'].ids, []);
  const hidden = plan('examples/vite-adapter-vue/package.json');
  assert.equal(hidden.full, true); assert.ok(hidden.affectedOwners.includes('js-plugins/postcss'));
  assert.ok(hidden.sets.ts.ids.some(id => id.includes('js-plugins/postcss/')));
  for (const file of ['examples/react/package.json', 'examples/react/farm.config.ts', 'js-plugins/postcss/src/index.ts']) {
    assert.equal(plan(file).full, true);
  }
  for (const file of ['packages/core/src/index.ts', 'packages/cli/src/index.ts', 'packages/runtime/src/index.ts',
    'packages/runtime-plugin-hmr/src/index.ts', 'packages/runtime-plugin-import-meta/src/index.ts', 'unknown-source.ts']) {
    const full = plan(file); assert.equal(full.full, true);
    for (const kind of ['ts', 'examples-build', 'examples-browser']) {
      assert.deepEqual(full.sets[kind].ids, inventory.items.filter(row => row.kind === kind).map(row => row.id).sort());
    }
  }
});
