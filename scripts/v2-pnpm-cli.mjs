import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';

// Resolve an already installed, pinned pnpm JS entry. Never invoke a shell shim,
// Corepack download or a package-manager installation as an implicit fallback.
export function resolvePnpmCli({ env = process.env, repositoryRoot = fileURLToPath(new URL('../', import.meta.url)), entry } = {}) {
  const expected = JSON.parse(fs.readFileSync(path.join(repositoryRoot, 'package.json'))).packageManager?.match(/^pnpm@(\d+\.\d+\.\d+)$/)?.[1];
  assert.ok(expected, 'Repository must declare an exact pnpm version');
  let candidate = entry ?? env.FARM_PNPM_CLI ?? env.npm_execpath;
  if (candidate == null && env.PNPM_HOME) {
    assert.ok(path.isAbsolute(env.PNPM_HOME), 'Installed PNPM_HOME must be absolute');
    candidate = path.resolve(env.PNPM_HOME, '../pnpm/bin/pnpm.cjs');
  }
  assert.ok(typeof candidate === 'string' && path.isAbsolute(candidate), 'Set FARM_PNPM_CLI to the installed pinned pnpm JS entry; npm_execpath is optional');
  const cli = fs.realpathSync(candidate);
  assert.ok(fs.statSync(cli).isFile() && path.basename(cli) === 'pnpm.cjs', 'Expected pnpm JS CLI, not a platform shell wrapper');
  const directory = path.dirname(path.dirname(cli));
  const manifest = JSON.parse(fs.readFileSync(path.join(directory, 'package.json')));
  assert.equal(manifest.name, 'pnpm', 'Untrusted package manager entry');
  assert.equal(manifest.version, expected, 'Installed pnpm differs from repository pin');
  assert.equal(manifest.bin?.pnpm, 'bin/pnpm.cjs', 'Unexpected pnpm entry contract');
  assert.equal(fs.realpathSync(path.join(directory, manifest.bin.pnpm)), cli);
  return cli;
}
export function probePnpmCli(options = {}) {
  const cli = resolvePnpmCli(options);
  const version = execFileSync(process.execPath, [cli, '--version'], { env: options.env ?? process.env, encoding: 'utf8', windowsHide: true, timeout: 10000 }).trim();
  assert.equal(version, JSON.parse(fs.readFileSync(path.join(path.dirname(path.dirname(cli)), 'package.json'))).version, 'pnpm executable version differs from its manifest');
  return { manager: 'pnpm', cli, version, lifecycleEntryPresent: Boolean((options.env ?? process.env).npm_execpath) };
}
