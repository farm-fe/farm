import assert from 'node:assert/strict';
import fs from 'node:fs';
import {test} from 'node:test';
const file = new URL('../website/vercel.json', import.meta.url);
const config = JSON.parse(fs.readFileSync(file, 'utf8'));
test('project-scoped Git rule disables only exact main', () => {
  assert.deepEqual(config.git, {deploymentEnabled:{main:false}});
  assert.notEqual(config.github?.enabled, false);
});
for (const branch of ['main', 'main-feature', 'master', 'codex/farm-docs-preview', 'v2-dev']) {
  test(`documented Git deployment scope: ${branch}`, () => {
    const enabled = config.git.deploymentEnabled[branch] ?? true;
    assert.equal(enabled, branch !== 'main');
  });
}
test('scope policy does not introduce promotion, deletion, credentials or a global integration switch', () => {
  assert.deepEqual(Object.keys(config).sort(), ['$schema','buildCommand','framework','git','installCommand','outputDirectory'].sort());
  assert.equal(config.installCommand, 'node ../scripts/farm-docs-vercel-build.mjs --install');
  assert.equal(config.buildCommand, 'node ../scripts/farm-docs-vercel-build.mjs');
});
