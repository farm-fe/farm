import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { resolvePnpmCli } from './v2-pnpm-cli.mjs';

test('resolver accepts only the installed pinned pnpm JS entry without executing or installing it', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-pnpm-entry-'));
  try {
    const repositoryRoot = path.join(directory, 'repo'), packageRoot = path.join(directory, 'installer/node_modules/pnpm');
    const cli = path.join(packageRoot, 'bin/pnpm.cjs'), home = path.join(directory, 'installer/node_modules/.bin');
    fs.mkdirSync(repositoryRoot); fs.mkdirSync(path.dirname(cli), { recursive: true }); fs.mkdirSync(home);
    fs.writeFileSync(path.join(repositoryRoot, 'package.json'), JSON.stringify({ packageManager: 'pnpm@9.4.0' }));
    const manifest = { name: 'pnpm', version: '9.4.0', bin: { pnpm: 'bin/pnpm.cjs' } };
    const writeManifest = value => fs.writeFileSync(path.join(packageRoot, 'package.json'), JSON.stringify(value));
    writeManifest(manifest); fs.writeFileSync(cli, 'throw Error("Resolver must never execute this fixture");');
    assert.equal(resolvePnpmCli({ repositoryRoot, env: { FARM_PNPM_CLI: cli } }), fs.realpathSync(cli));
    assert.equal(resolvePnpmCli({ repositoryRoot, env: { PNPM_HOME: home } }), fs.realpathSync(cli));
    assert.equal(resolvePnpmCli({ repositoryRoot, env: { npm_execpath: cli } }), fs.realpathSync(cli));
    assert.throws(() => resolvePnpmCli({ repositoryRoot, env: {} }), /Set FARM_PNPM_CLI/);
    assert.throws(() => resolvePnpmCli({ repositoryRoot, env: { FARM_PNPM_CLI: 'pnpm.cjs' } }));
    for (const value of [{ ...manifest, name: 'npm' }, { ...manifest, version: '9.5.0' }, { ...manifest, bin: { pnpm: 'other.cjs' } }]) {
      writeManifest(value); assert.throws(() => resolvePnpmCli({ repositoryRoot, entry: cli, env: {} }));
    }
    writeManifest(manifest);
    const shim = path.join(packageRoot, 'bin/pnpm.cmd'); fs.writeFileSync(shim, 'exit 1');
    assert.throws(() => resolvePnpmCli({ repositoryRoot, entry: shim, env: {} }), /shell wrapper/);
  } finally { fs.rmSync(directory, { recursive: true, force: true }); }
});

test('both real candidate entrypoints run via pnpm exec without npm_execpath', () => {
  const root = fileURLToPath(new URL('../', import.meta.url));
  const env = { ...process.env }; delete env.npm_execpath;
  const pnpm = resolvePnpmCli({ env }); env.FARM_PNPM_CLI = pnpm;
  const expectedVersion = JSON.parse(fs.readFileSync(path.join(root, 'package.json'))).packageManager.slice('pnpm@'.length);
  for (const entry of ['prepare-v2-candidate-js.mjs', 'pack-v2-npm-candidates.mjs']) {
    const output = execFileSync(process.execPath, [pnpm, 'exec', process.execPath, fileURLToPath(new URL(entry, import.meta.url)), '--check-package-manager'], { cwd: root, env, encoding: 'utf8', windowsHide: true, timeout: 30000, maxBuffer: 1024 * 1024 });
    const result = JSON.parse(output.trim().split(/\r?\n/).at(-1));
    assert.equal(result.manager, 'pnpm'); assert.equal(result.version, expectedVersion);
    assert.equal(result.lifecycleEntryPresent, false);
    assert.equal(fs.realpathSync(result.cli), pnpm);
  }
});
