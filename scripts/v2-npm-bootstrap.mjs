// cspell:words userconfig globalconfig
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { PUBLISHER_NODE_VERSION, PUBLISHER_NPM_VERSION, resolveNpmCli } from './v2-npm-cli.mjs';

// This bootstrap is intentionally separate from the credential-bearing publisher.
// All subprocesses use the selected Node executable and a fresh environment, not
// a shell/npm wrapper, project configuration, ambient auth or OIDC credentials.
export function bootstrapPublisherNpm({ nodeExecutable = process.execPath, platform = process.platform,
  run = spawnSync, temporaryRoot = os.tmpdir() } = {}) {
  assert.ok(path.isAbsolute(nodeExecutable), 'Node executable must be absolute');
  const node = fs.realpathSync(nodeExecutable);
  const bundled = resolveNpmCli({ nodeExecutable: node, platform, requireSelectedInstall: true });
  const prefix = platform === 'win32' ? path.dirname(node) : path.resolve(path.dirname(node), '..');
  const scratch = fs.mkdtempSync(path.join(temporaryRoot, 'farm-npm-bootstrap-'));
  try {
    const home = path.join(scratch, 'home');
    const cwd = path.join(scratch, 'work');
    const temp = path.join(scratch, 'tmp');
    for (const directory of [home, cwd, temp]) fs.mkdirSync(directory, { mode: 0o700 });
    const userconfig = path.join(scratch, 'user.npmrc');
    const globalconfig = path.join(scratch, 'global.npmrc');
    fs.writeFileSync(userconfig, 'registry=https://registry.npmjs.org/\n', { mode: 0o600 });
    fs.writeFileSync(globalconfig, '', { mode: 0o600 });
    const env = {
      HOME: home, USERPROFILE: home, TMPDIR: temp, TMP: temp, TEMP: temp, CI: 'true',
      NPM_CONFIG_USERCONFIG: userconfig, NPM_CONFIG_GLOBALCONFIG: globalconfig,
      npm_config_cache: path.join(scratch, 'cache'), npm_config_ignore_scripts: 'true',
      npm_config_update_notifier: 'false'
    };
    const invoke = (args, timeout = 15_000) => {
      let result;
      try {
        result = run(node, args, { cwd, env: { ...env }, encoding: 'utf8', timeout,
          maxBuffer: 64 * 1024, shell: false });
      } catch {
        throw new Error('Controlled npm bootstrap command failed; child output withheld');
      }
      assert.ok(result && !result.error && result.status === 0,
        'Controlled npm bootstrap command failed; child output withheld');
      return result.stdout;
    };
    assert.ok(invoke(['--version']).trim() === `v${PUBLISHER_NODE_VERSION}`, 'Unexpected controlled Node version');
    invoke([bundled, 'install', '--global', '--prefix', prefix, `npm@${PUBLISHER_NPM_VERSION}`,
      '--ignore-scripts', '--no-audit', '--no-fund', '--no-progress',
      '--registry=https://registry.npmjs.org/'], 300_000);
    // Re-resolve after installation. Do not retain an entry or version merely
    // because the old manager reported a successful installation.
    const cli = resolveNpmCli({ nodeExecutable: node, platform,
      expectedVersion: PUBLISHER_NPM_VERSION, requireSelectedInstall: true });
    assert.ok(invoke([cli, '--version']).trim() === PUBLISHER_NPM_VERSION, 'Installed npm CLI version differs');
    return { node, cli, version: PUBLISHER_NPM_VERSION };
  } finally {
    fs.rmSync(scratch, { recursive: true, force: true });
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(fs.realpathSync(process.argv[1])).href) {
  const result = bootstrapPublisherNpm();
  assert.ok(!/[\r\n]/.test(result.node), 'Invalid selected Node executable path');
  if (process.env.GITHUB_OUTPUT) fs.appendFileSync(process.env.GITHUB_OUTPUT, `node-executable=${result.node}\n`);
  console.log(`Verified controlled npm ${result.version} on Node ${PUBLISHER_NODE_VERSION}`);
}
