// cspell:words nonpublisher userconfig globalconfig
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import fsSync from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { PUBLISHER_NODE_VERSION, PUBLISHER_NPM_VERSION, resolveNpmCli, resolvePublisherNpmCli } from './v2-npm-cli.mjs';
import { bootstrapPublisherNpm } from './v2-npm-bootstrap.mjs';

async function fixture(platform) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-bundled-npm-'));
  const node = path.join(root, platform === 'win32' ? 'node.exe' : 'bin/node');
  const npm = path.join(root, platform === 'win32' ? 'node_modules/npm' : 'lib/node_modules/npm');
  await fs.mkdir(path.dirname(node), { recursive: true }); await fs.writeFile(node, 'fixture only');
  await fs.mkdir(path.join(npm, 'bin'), { recursive: true });
  const manifest = { name: 'npm', version: '10.9.3', bin: { npm: 'bin/npm-cli.js' } };
  await fs.writeFile(path.join(npm, 'package.json'), JSON.stringify(manifest));
  const cli = path.join(npm, 'bin/npm-cli.js'); await fs.writeFile(cli, '// fixture only');
  return { root, node, npm, manifest, cli };
}

test('Windows and Unix Node layouts resolve only their bundled npm JavaScript entry', async () => {
  for (const platform of ['win32', 'linux', 'darwin']) {
    const f = await fixture(platform);
    try { assert.equal(resolveNpmCli({ platform, nodeExecutable: f.node }), await fs.realpath(f.cli)); }
    finally { await fs.rm(f.root, { recursive: true }); }
  }
});

test('missing layout, wrong identity, changed bin and escaped realpath fail closed', async () => {
  const f = await fixture('linux');
  try {
    assert.throws(() => resolveNpmCli({ platform: 'win32', nodeExecutable: f.node }));
    for (const manifest of [{ ...f.manifest, name: 'other' }, { ...f.manifest, version: 'unknown' }, { ...f.manifest, bin: { npm: '../other.js' } }]) {
      await fs.writeFile(path.join(f.npm, 'package.json'), JSON.stringify(manifest));
      assert.throws(() => resolveNpmCli({ platform: 'linux', nodeExecutable: f.node }));
    }
    await fs.writeFile(path.join(f.npm, 'package.json'), JSON.stringify(f.manifest));
    const outside = path.join(f.root, 'outside'); await fs.mkdir(outside);
    await fs.writeFile(path.join(outside, 'npm-cli.js'), '// outside package');
    await fs.rm(path.join(f.npm, 'bin'), { recursive: true });
    await fs.symlink(outside, path.join(f.npm, 'bin'), process.platform === 'win32' ? 'junction' : 'dir');
    assert.throws(() => resolveNpmCli({ platform: 'linux', nodeExecutable: f.node }), /stay inside/);
  } finally { await fs.rm(f.root, { recursive: true }); }
});

test('actual CI Node distribution exposes the verified bundled manager without a shell lookup', () => {
  assert.ok(path.isAbsolute(resolveNpmCli()));
});


test('publisher pin fails closed without changing nonpublisher version compatibility', async () => {
  const f = await fixture('linux');
  try {
    for (const version of ['10.9.3', '11.20.0', '11.21.0', '11.21.2-beta.1', '11.21.3']) {
      await fs.writeFile(path.join(f.npm, 'package.json'), JSON.stringify({ ...f.manifest, version }));
      assert.equal(resolveNpmCli({ nodeExecutable: f.node }), f.cli);
      assert.throws(() => resolveNpmCli({ nodeExecutable: f.node, expectedVersion: PUBLISHER_NPM_VERSION,
        requireSelectedInstall: true }), /controlled npm version/);
    }
    await fs.writeFile(path.join(f.npm, 'package.json'), JSON.stringify({ ...f.manifest, version: PUBLISHER_NPM_VERSION }));
    assert.equal(resolveNpmCli({ nodeExecutable: f.node, expectedVersion: PUBLISHER_NPM_VERSION,
      requireSelectedInstall: true }), f.cli);
    if (process.versions.node !== PUBLISHER_NODE_VERSION) assert.throws(() => resolvePublisherNpmCli(), /controlled Node version/);
  } finally { await fs.rm(f.root, { recursive: true }); }
});

test('strict selected-install boundary rejects an npm package root redirected by symlink', async () => {
  const f = await fixture('linux');
  try {
    const outside = path.join(f.root, 'outside-npm');
    await fs.rename(f.npm, outside);
    await fs.symlink(outside, f.npm, process.platform === 'win32' ? 'junction' : 'dir');
    assert.throws(() => resolveNpmCli({ nodeExecutable: f.node, requireSelectedInstall: true }), /selected Node installation/);
    assert.throws(() => bootstrapPublisherNpm({ nodeExecutable: f.node, temporaryRoot: f.root,
      run: () => { throw new Error('Must not execute escaped manager'); } }), /selected Node installation/);
  } finally { await fs.rm(f.root, { recursive: true }); }
});

function successfulBootstrap(f, calls, { nodeVersion = PUBLISHER_NODE_VERSION,
  manifestVersion = PUBLISHER_NPM_VERSION, cliVersion = PUBLISHER_NPM_VERSION } = {}) {
  return (node, args, options) => {
    calls.push({ node, args, options });
    if (args[0] === '--version') return { status: 0, stdout: `v${nodeVersion}\n` };
    if (args[1] === 'install') {
      fsSync.writeFileSync(path.join(f.npm, 'package.json'), JSON.stringify({ ...f.manifest, version: manifestVersion }));
      return { status: 0, stdout: 'Installed fixture only' };
    }
    assert.deepEqual(args, [f.cli, '--version']);
    return { status: 0, stdout: `${cliVersion}\n` };
  };
}

test('bootstrap pins npm inside selected Node with isolated credential-free commands and no lifecycle hooks', async () => {
  const hostile = { ACTIONS_ID_TOKEN_REQUEST_URL: 'https://example.invalid/oidc',
    ACTIONS_ID_TOKEN_REQUEST_TOKEN: 'oidc-secret', NPM_TOKEN: 'npm-secret', NODE_AUTH_TOKEN: 'node-auth-secret',
    GH_TOKEN: 'github-secret', NODE_OPTIONS: '--require malicious.cjs', npm_execpath: '/malicious/npm',
    NPM_CONFIG_REGISTRY: 'https://example.invalid/npm', npm_config_userconfig: '/malicious/.npmrc',
    npm_config_script_shell: '/malicious/shell', PATH: '/malicious/bin' };
  const saved = Object.fromEntries(Object.keys(hostile).map(key => [key, process.env[key]]));
  Object.assign(process.env, hostile);
  try {
    for (const platform of ['linux', 'win32']) {
      const f = await fixture(platform);
      const calls = [];
      try {
        const fakeRun = successfulBootstrap(f, calls);
        const result = bootstrapPublisherNpm({ nodeExecutable: f.node, platform, temporaryRoot: f.root,
          run(node, args, options) {
            assert.equal(options.shell, false);
            assert.equal(options.maxBuffer, 64 * 1024);
            assert.equal(node, f.node);
            assert.ok(path.isAbsolute(options.cwd));
            assert.notEqual(options.cwd, process.cwd());
            assert.notEqual(options.cwd, options.env.HOME);
            assert.deepEqual(fsSync.readdirSync(options.cwd), []);
            assert.equal(fsSync.readFileSync(options.env.NPM_CONFIG_USERCONFIG, 'utf8'), 'registry=https://registry.npmjs.org/\n');
            assert.equal(fsSync.readFileSync(options.env.NPM_CONFIG_GLOBALCONFIG, 'utf8'), '');
            assert.equal(options.env.npm_config_ignore_scripts, 'true');
            assert.equal(options.env.npm_config_update_notifier, 'false');
            for (const [key, value] of Object.entries(hostile)) assert.notEqual(options.env[key], value, key);
            assert.equal(Object.hasOwn(options.env, 'PATH'), false);
            assert.equal(Object.hasOwn(options.env, 'GITHUB_ACTIONS'), false);
            for (const key of Object.keys(options.env)) assert.doesNotMatch(key, /TOKEN|AUTH|NODE_OPTIONS|execpath/i);
            return fakeRun(node, args, options);
          }
        });
        assert.deepEqual(result, { node: f.node, cli: f.cli, version: PUBLISHER_NPM_VERSION });
        assert.equal(calls.length, 3);
        assert.deepEqual(calls[1].args, [f.cli, 'install', '--global', '--prefix', f.root,
          'npm@11.21.2', '--ignore-scripts', '--no-audit', '--no-fund', '--no-progress',
          '--registry=https://registry.npmjs.org/']);
        assert.equal(calls[1].options.timeout, 300_000);
        assert.equal(calls[0].options.timeout, 15_000);
        await assert.rejects(fs.access(calls[0].options.cwd), /ENOENT/);
        assert.equal((await fs.readdir(f.root)).some(name => name.startsWith('farm-npm-bootstrap-')), false);
      } finally { await fs.rm(f.root, { recursive: true }); }
    }
  } finally {
    for (const [key, value] of Object.entries(saved)) {
      if (value === undefined) delete process.env[key]; else process.env[key] = value;
    }
  }
});

test('bootstrap rejects wrong Node before install and revalidates manifest and executable versions after install', async () => {
  for (const [change, message, count] of [
    [{ nodeVersion: '22.19.0' }, /controlled Node version/, 1],
    [{ manifestVersion: '10.9.3' }, /controlled npm version/, 2],
    [{ manifestVersion: '11.21.3' }, /controlled npm version/, 2],
    [{ cliVersion: '11.20.0' }, /Installed npm CLI version differs/, 3],
    [{ cliVersion: 'untrusted-stdout-secret' }, /Installed npm CLI version differs/, 3]
  ]) {
    const f = await fixture('linux');
    const calls = [];
    try {
      assert.throws(() => bootstrapPublisherNpm({ nodeExecutable: f.node, temporaryRoot: f.root,
        run: successfulBootstrap(f, calls, change) }), error => {
        assert.match(error.message, message);
        assert.doesNotMatch(String(error), /untrusted-stdout-secret/);
        return true;
      });
      assert.equal(calls.length, count);
      await assert.rejects(fs.access(calls[0].options.cwd), /ENOENT/);
    } finally { await fs.rm(f.root, { recursive: true }); }
  }
});

test('post-install resolution rejects a replaced package, manifest or CLI escaping its boundary', async () => {
  for (const escape of ['package', 'manifest', 'cli']) {
    const f = await fixture('linux');
    const calls = [];
    try {
      const fakeRun = successfulBootstrap(f, calls);
      assert.throws(() => bootstrapPublisherNpm({ nodeExecutable: f.node, temporaryRoot: f.root,
        run(node, args, options) {
          const result = fakeRun(node, args, options);
          if (args[1] === 'install') {
            const target = escape === 'package' ? f.npm : escape === 'manifest' ? path.join(f.npm, 'package.json') : f.cli;
            const outside = path.join(f.root, 'outside');
            fsSync.renameSync(target, outside);
            fsSync.symlinkSync(outside, target, escape === 'package' ? 'dir' : 'file');
          }
          return result;
        }
      }), /stay inside/);
      assert.equal(calls.length, 2);
      await assert.rejects(fs.access(calls[0].options.cwd), /ENOENT/);
    } finally { await fs.rm(f.root, { recursive: true }); }
  }
});

test('bootstrap failures and timeouts withhold child output and remove isolated files', async () => {
  for (const stage of [0, 1, 2]) {
    for (const failure of [
      () => ({ status: 1, stdout: 'untrusted-stdout-secret', stderr: 'untrusted-stderr-secret' }),
      () => ({ status: null, error: Object.assign(new Error('untrusted-error-secret'), { code: 'ETIMEDOUT' }) }),
      () => { throw new Error('untrusted-thrown-secret'); }
    ]) {
      const f = await fixture('linux');
      const calls = [];
      let cwd;
      try {
        const fakeRun = successfulBootstrap(f, calls);
        assert.throws(() => bootstrapPublisherNpm({ nodeExecutable: f.node, temporaryRoot: f.root,
          run(node, args, options) {
            cwd = options.cwd;
            if (calls.length === stage) return failure();
            return fakeRun(node, args, options);
          }
        }), error => {
          assert.match(error.message, /command failed; child output withheld/);
          assert.doesNotMatch(String(error), /untrusted-/);
          return true;
        });
        await assert.rejects(fs.access(cwd), /ENOENT/);
      } finally { await fs.rm(f.root, { recursive: true }); }
    }
  }
});

test('incremental publisher alone gains npm OIDC permissions and bootstraps before receiving candidate data', async () => {
  const workflow = await fs.readFile(new URL('../.github/workflows/incremental-release.yaml', import.meta.url), 'utf8');
  const publisher = workflow.split('\n  publisher:\n')[1].split('\n  public-proof:\n')[0];
  assert.match(publisher, /permissions:\n      contents: write\n      actions: read\n      id-token: write/);
  assert.match(publisher, /node-version: 22\.20\.0/);
  assert.doesNotMatch(publisher, /NPM_TOKEN|NODE_AUTH_TOKEN|registry-url|npm install|npx /);
  assert.match(publisher, /ACTIONS_ID_TOKEN_REQUEST_URL: ''\n          ACTIONS_ID_TOKEN_REQUEST_TOKEN: ''/);
  assert.match(publisher, /FARM_NODE_EXECUTABLE: \$\{\{ steps\.publisher-npm\.outputs\.node-executable \}\}/);
  assert.match(publisher, /\/usr\/bin\/env -i GITHUB_OUTPUT="\$GITHUB_OUTPUT" "\$node_executable" "\$GITHUB_WORKSPACE\/scripts\/v2-npm-bootstrap\.mjs"/);
  assert.ok(publisher.indexOf('scripts/v2-npm-bootstrap.mjs') < publisher.indexOf('uses: actions/download-artifact@v4'));
  assert.ok(publisher.indexOf('scripts/v2-npm-bootstrap.mjs') < publisher.indexOf('incremental-release-publish.py upload'));
  assert.equal((publisher.match(/id-token: write/g) ?? []).length, 1);
  assert.doesNotMatch(workflow.split('\njobs:\n')[0], /id-token:/);
  for (const name of ['source', 'verify', 'producer', 'public-proof', 'finalize']) {
    const section = workflow.split(`\n  ${name}:\n`)[1].split(/\n  [a-z][a-z-]*:\n/)[0];
    assert.doesNotMatch(section, /id-token:/);
  }
});
