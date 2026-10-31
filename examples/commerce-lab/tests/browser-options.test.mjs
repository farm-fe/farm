import test from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import { browserOptions } from './browser-options.mjs';

test('default acceptance retains all three groups and existing entrypoints', () => {
  const options = browserOptions({});
  assert.deepEqual(options.groups, ['js', 'rust', 'mixed']);
  assert.deepEqual(options.buildArgs, ['scripts/run.mjs', 'build']);
  assert.equal(options.configFor('rust'), 'farm.rust.config.ts');
});
test('explicit config is passed identically to npm build and preview', () => {
  const file = path.resolve('generated config.ts');
  const options = browserOptions({ COMMERCE_BROWSER_GROUPS: 'js', COMMERCE_CONFIG_OVERRIDE: file, COMMERCE_NPM_CLI: 'npm-cli.js' });
  assert.deepEqual(options.groups, ['js']);
  assert.deepEqual(options.buildArgs, ['npm-cli.js', 'run', 'build', '--', '--config', file]);
  assert.equal(options.configFor('js'), file);
});
test('direct runner receives the same explicit config without npm separator', () => {
  const file = path.resolve('generated.ts');
  assert.deepEqual(browserOptions({ COMMERCE_CONFIG_OVERRIDE: file }).buildArgs, ['../../packages/cli/bin/farm.mjs', 'build', '--config', file]);
});
test('isolated npm project avoids the default runner config argument', () => {
  const project = path.resolve('isolated-driver'), file = path.join(project, 'farm.config.ts');
  const options = browserOptions({ COMMERCE_NPM_CLI: 'npm-cli.js', COMMERCE_NPM_PROJECT: project, COMMERCE_CONFIG_OVERRIDE: file });
  assert.deepEqual(options.buildArgs, ['npm-cli.js', '--prefix', project, 'run', 'build', '--', '--config', file]);
  assert.throws(() => browserOptions({ COMMERCE_NPM_PROJECT: './relative' }));
});
test('invalid groups, duplicates and relative overrides are rejected before build', () => {
  for (const value of ['', 'js,js', 'other', 'js,']) assert.throws(() => browserOptions({ COMMERCE_BROWSER_GROUPS: value }));
  assert.throws(() => browserOptions({ COMMERCE_CONFIG_OVERRIDE: './generated.ts' }));
});
