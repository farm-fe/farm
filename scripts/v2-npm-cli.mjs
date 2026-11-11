import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

export const PUBLISHER_NODE_VERSION = '22.20.0';
export const PUBLISHER_NPM_VERSION = '11.21.2';

// Only use npm in the selected Node installation. Never search PATH, invoke
// shell wrappers or trust npm_execpath as an override. Nonpublishing callers may
// keep using the distribution's bundled npm; the publisher requires our pin.
export function resolveNpmCli({ nodeExecutable = process.execPath, platform = process.platform,
  expectedVersion, requireSelectedInstall = false } = {}) {
  assert.ok(typeof nodeExecutable === 'string' && path.isAbsolute(nodeExecutable), 'Node executable must be absolute');
  const executable = fs.realpathSync(nodeExecutable);
  assert.ok(fs.statSync(executable).isFile(), 'Node executable must be a regular file');
  const directory = path.dirname(executable);
  const expectedRoot = platform === 'win32' ? path.join(directory, 'node_modules', 'npm')
    : path.resolve(directory, '..', 'lib', 'node_modules', 'npm');
  const root = fs.realpathSync(expectedRoot);
  if (requireSelectedInstall) assert.equal(root, expectedRoot, 'npm must stay inside the selected Node installation');
  const manifestFile = fs.realpathSync(path.join(root, 'package.json'));
  assert.equal(manifestFile, path.join(root, 'package.json'), 'npm manifest must stay inside its package');
  const manifest = JSON.parse(fs.readFileSync(manifestFile, 'utf8'));
  assert.equal(manifest.name, 'npm', 'Untrusted bundled package manager identity');
  assert.match(manifest.version ?? '', /^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/, 'Invalid bundled npm version');
  if (expectedVersion !== undefined) assert.equal(manifest.version, expectedVersion, 'Unexpected controlled npm version');
  assert.equal(manifest.bin?.npm, 'bin/npm-cli.js', 'Unexpected bundled npm entry contract');
  const cli = fs.realpathSync(path.join(root, manifest.bin.npm));
  assert.equal(path.relative(root, cli).replaceAll('\\', '/'), 'bin/npm-cli.js', 'npm entry must stay inside its package');
  assert.ok(fs.statSync(cli).isFile(), 'npm entry must be a regular JavaScript file');
  return cli;
}

export function resolvePublisherNpmCli() {
  assert.equal(process.versions.node, PUBLISHER_NODE_VERSION, 'Unexpected controlled Node version');
  return resolveNpmCli({ expectedVersion: PUBLISHER_NPM_VERSION, requireSelectedInstall: true });
}
