import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { candidateJsBuilds } from './prepare-v2-candidate-js.mjs';
test('candidate producer builds public JS in dependency order and never recompiles native wrappers', () => {
  const plan = JSON.parse(fs.readFileSync(new URL('../docs/releases/v2-candidate.json', import.meta.url)));
  const recipe = candidateJsBuilds(plan);
  assert.equal(new Set(recipe).size, recipe.length);
  assert.ok(recipe.indexOf('packages/core') < recipe.indexOf('js-plugins/babel'));
  assert.ok(recipe.indexOf('js-plugins/babel') < recipe.indexOf('js-plugins/react-compiler'));
  assert.ok(!recipe.some(dir => dir.startsWith('rust-plugins/') || dir === 'packages/create-farm'));
  for (const directory of recipe) {
    const manifest = JSON.parse(fs.readFileSync(new URL(`../${directory}/package.json`, import.meta.url)));
    assert.doesNotMatch(manifest.scripts.build, /\bcargo\b|\bnapi\b|farm-plugin-tools build/);
  }
});
