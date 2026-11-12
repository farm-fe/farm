// cspell:words micromatch picomatch readdirp
import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import semver from 'semver';
import { parse } from 'yaml';

const core = JSON.parse(fs.readFileSync(new URL('../packages/core/package.json', import.meta.url)));
const lock = parse(fs.readFileSync(new URL('../pnpm-lock.yaml', import.meta.url), 'utf8'));
const dependencies = lock.importers['packages/core'].dependencies;

function resolved(chain) {
  let version = dependencies[chain[0]].version;
  for (let i = 1; i < chain.length; i++) {
    const snapshot = lock.snapshots[`${chain[i - 1]}@${version}`];
    assert.ok(snapshot, `Missing lock snapshot for ${chain[i - 1]}@${version}`);
    version = snapshot.dependencies[chain[i]];
    assert.ok(version, `Missing dependency ${chain[i]}`);
  }
  return version.split('(')[0];
}

test('Core ws range excludes the known vulnerable release line', () => {
  assert.ok(semver.gte(semver.minVersion(core.dependencies.ws), '8.21.0'));
});

for (const [chain, minimum] of [
  [['ws'], '8.21.0'],
  [['http-proxy', 'follow-redirects'], '1.16.0'],
  [['execa', 'cross-spawn'], '7.0.5'],
  [['fast-glob', 'micromatch'], '4.0.8'],
  [['fast-glob', 'micromatch', 'picomatch'], '2.3.2'],
  [['fast-glob', 'micromatch', 'braces'], '3.0.3'],
  [['anymatch', 'picomatch'], '2.3.2'],
  [['chokidar', 'anymatch', 'picomatch'], '2.3.2'],
  [['chokidar', 'readdirp', 'picomatch'], '2.3.2'],
  [['chokidar', 'braces'], '3.0.3']
]) {
  test(`Core build lock resolves patched ${chain.join(' > ')}`, () => {
    const version = resolved(chain);
    assert.ok(semver.gte(version, minimum), `${chain.join(' > ')} resolves ${version}; requires >=${minimum}`);
  });
}
