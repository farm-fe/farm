import assert from 'node:assert/strict';
import { realpathSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
import { createRequire } from 'node:module';
import { resolveNpmCli } from './v2-npm-cli.mjs';

// The publisher installs Node, not workspace dependencies. Use only its bundled npm.
const npmRequire = createRequire(resolveNpmCli());
const semver = npmRequire('../node_modules/semver');

function exactVersion(value) {
  assert.equal(typeof value, 'string', 'Registry version must be a string');
  const parsed = new semver.SemVer(value);
  const canonical = parsed.version + (parsed.build.length ? `+${parsed.build.join('.')}` : '');
  assert.equal(value, canonical, 'Exact SemVer required');
  return parsed;
}

export function compareVersions(left, right) {
  return exactVersion(left).compare(exactVersion(right));
}

export function compareRegistryLatest(latest, target) {
  const wanted = exactVersion(target);
  assert.equal(wanted.prerelease.length, 0, 'Stable publication target required');
  assert.equal(wanted.build.length, 0, 'Exact stable publication target required');
  return latest === null ? null : exactVersion(latest).compare(wanted);
}

if (process.argv[1] && import.meta.url === pathToFileURL(realpathSync(process.argv[1])).href) {
  const [latest, target] = JSON.parse(process.argv[2]);
  console.log(JSON.stringify(compareRegistryLatest(latest, target)));
}
