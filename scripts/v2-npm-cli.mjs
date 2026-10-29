import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

// Only use npm bundled with the selected Node installation. Never search PATH,
// invoke shell wrappers, download a manager or trust npm_execpath as an override.
export function resolveNpmCli({ nodeExecutable = process.execPath, platform = process.platform } = {}) {
  assert.ok(typeof nodeExecutable === 'string' && path.isAbsolute(nodeExecutable), 'Node executable must be absolute');
  const executable = fs.realpathSync(nodeExecutable);
  assert.ok(fs.statSync(executable).isFile(), 'Node executable must be a regular file');
  const directory = path.dirname(executable);
  const expectedRoot = platform === 'win32' ? path.join(directory, 'node_modules', 'npm')
    : path.resolve(directory, '..', 'lib', 'node_modules', 'npm');
  const root = fs.realpathSync(expectedRoot);
  const manifestFile = fs.realpathSync(path.join(root, 'package.json'));
  assert.equal(manifestFile, path.join(root, 'package.json'), 'npm manifest must stay inside its package');
  const manifest = JSON.parse(fs.readFileSync(manifestFile, 'utf8'));
  assert.equal(manifest.name, 'npm', 'Untrusted bundled package manager identity');
  assert.match(manifest.version ?? '', /^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/, 'Invalid bundled npm version');
  assert.equal(manifest.bin?.npm, 'bin/npm-cli.js', 'Unexpected bundled npm entry contract');
  const cli = fs.realpathSync(path.join(root, manifest.bin.npm));
  assert.equal(path.relative(root, cli).replaceAll('\\', '/'), 'bin/npm-cli.js', 'npm entry must stay inside its package');
  assert.ok(fs.statSync(cli).isFile(), 'npm entry must be a regular JavaScript file');
  return cli;
}
