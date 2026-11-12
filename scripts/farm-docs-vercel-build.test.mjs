import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';
import { deploymentTarget, deploymentPlan, releaseManifests, preflight, buildPublishedDocs, assertRegistryLock } from './farm-docs-vercel-build.mjs';

const repository = fileURLToPath(new URL('../', import.meta.url));
const read = file => JSON.parse(fs.readFileSync(file));
const integrity = 'sha512-' + Buffer.alloc(64, 7).toString('base64');
const target = 'linux-x64-gnu';
const clone = value => JSON.parse(JSON.stringify(value));

function isInside(directory, file, paths = path) {
  const relative = paths.relative(directory, file);
  return relative !== '' && relative !== '..' && !relative.startsWith('..' + paths.sep) && !paths.isAbsolute(relative);
}

test('path containment handles Windows casing and rejects sibling prefixes', () => {
  assert.ok(isInside('C:\\Temp\\Consumer', 'c:\\temp\\consumer\\node_modules\\cli.mjs', path.win32));
  assert.ok(!isInside('C:\\Temp\\Consumer', 'C:\\Temp\\Consumer-other\\cli.mjs', path.win32));
  assert.ok(!isInside('C:\\Temp\\Consumer', 'D:\\Temp\\Consumer\\cli.mjs', path.win32));
});

function fixture(t) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-docs-build-test-'));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const put = (name, body) => { const file = path.join(root, name); fs.mkdirSync(path.dirname(file), { recursive: true }); fs.writeFileSync(file, body); };
  const manifests = new Map();
  const pkg = (folder, manifest) => { put(folder + '/package.json', JSON.stringify(manifest)); manifests.set(manifest.name, manifest); };
  pkg('packages/core', { name: '@farmfe/core', version: '2.0.0', main: 'dist/index.js', optionalDependencies: { '@farmfe/core-linux-x64-gnu': '2.0.0' } });
  pkg('packages/core/npm/linux-x64-gnu', { name: '@farmfe/core-linux-x64-gnu', version: '2.0.0', os: ['linux'], cpu: ['x64'], main: 'farm.linux-x64-gnu.node' });
  pkg('packages/cli', { name: '@farmfe/cli', version: '2.0.0', exports: { './package.json': './package.json' }, bin: { farm: './bin/farm.mjs' } });
  pkg('packages/docs', { name: '@farmfe/docs', version: '0.0.1', main: 'dist/index.js', dependencies: { '@farmfe/core': 'workspace:*', '@farmfe/plugin-docs': 'workspace:*' } });
  pkg('rust-plugins/docs', { name: '@farmfe/plugin-docs', version: '0.0.1', main: 'scripts/index.js', optionalDependencies: { '@farmfe/plugin-docs-linux-x64-gnu': '0.0.1' } });
  pkg('rust-plugins/docs/npm/linux-x64-gnu', { name: '@farmfe/plugin-docs-linux-x64-gnu', version: '0.0.1', os: ['linux'], cpu: ['x64'], main: 'index.farm' });
  const website = { dependencies: { '@farmfe/docs': 'workspace:*', react: '^18.2.0' }, devDependencies: { '@farmfe/core': 'workspace:*', '@farmfe/cli': 'workspace:*' } };
  put('website/package.json', JSON.stringify(website));
  put('website/farm.config.ts', '// test fixture');
  put('website/static/THIRD_PARTY_NOTICES.txt', 'original notice');
  put('website/src/page.tsx', '// test source');
  put('pnpm-workspace.yaml', 'packages: [packages/*, rust-plugins/*, website]');
  const metadata = new Map([...manifests].map(([name, manifest]) => {
    const meta = clone(manifest);
    for (const field of ['dependencies', 'optionalDependencies', 'peerDependencies']) for (const name of Object.keys(meta[field] ?? {})) if (name.startsWith('@farmfe/')) meta[field][name] = manifests.get(name).version;
    meta.dist = { integrity, tarball: `https://registry.npmjs.org/${name}/-/fixture.tgz` };
    return [name, meta];
  }));
  const calls = [];
  const hooks = {
    getMetadata: async name => clone(metadata.get(name)),
    execute: async (command, args, options) => {
      calls.push({ command, args, ...options });
      const write = (name, bytes) => { const file = path.join(options.cwd, name); fs.mkdirSync(path.dirname(file), { recursive: true }); fs.writeFileSync(file, bytes); };
      if (args.includes('--lockfile-only')) {
        write('pnpm-lock.yaml', `lockfileVersion: '9.0'\n# ${integrity}\n`);
      } else if (args.includes('--frozen-lockfile')) {
        for (const [name, manifest] of manifests) {
          write(`node_modules/${name}/package.json`, JSON.stringify(metadata.get(name)));
          if (manifest.main) write(`node_modules/${name}/${manifest.main}`, manifest.os ? '\x7fELFsynthetic native test only' : '// synthetic test only');
          if (manifest.bin) write(`node_modules/${name}/bin/farm.mjs`, '// synthetic never executed');
        }
        fs.mkdirSync(path.join(options.cwd, 'node_modules/@farmfe/core/binding'));
      } else if (args.includes('build')) {
        write('build-farm/index.html', '<a href="/THIRD_PARTY_NOTICES.txt">Notices</a>');
        write('build-farm/THIRD_PARTY_NOTICES.txt', 'original notice');
      }
    }
  };
  return { root, put, website, manifests, metadata, calls, hooks, options: { root, target, environment: {} } };
}

test('real manifest plan pins current v2 closure and matching precompiled target, never workspace or latest', () => {
  const manifests = releaseManifests();
  for (const target of ['linux-x64-gnu', 'darwin-arm64', 'win32-x64-msvc']) {
    const plan = deploymentPlan(read(path.join(repository, 'website/package.json')), manifests, target);
    assert.equal(plan.manifest.dependencies['@farmfe/core'], '2.0.0');
    assert.equal(plan.manifest.dependencies['@farmfe/docs'], '0.0.1');
    assert.equal(plan.manifest.dependencies['@emotion/is-prop-valid'], '1.4.0');
    assert.ok(plan.selected.has('@farmfe/core-' + target));
    for (const pkg of plan.selected.values()) if (pkg.os) assert.ok(pkg.name.endsWith('-' + target));
    assert.doesNotMatch(JSON.stringify(plan.manifest), /workspace:|latest|overrides|patches/);
  }
});

test('target selects native libc and rejects unsupported architectures', () => {
  assert.equal(deploymentTarget('linux', 'x64', '2.39'), 'linux-x64-gnu');
  assert.equal(deploymentTarget('linux', 'arm64', ''), 'linux-arm64-musl');
  assert.equal(deploymentTarget('darwin', 'arm64'), 'darwin-arm64');
  assert.throws(() => deploymentTarget('freebsd', 'x64'), /UNSUPPORTED_TARGET/);
});

test('preflight and Vercel install only read registry, without installs, source builds or outputs', async t => {
  const f = fixture(t);
  for (const phase of ['preflight', 'install']) {
    const result = await buildPublishedDocs({ ...f.options, phase }, f.hooks);
    assert.equal(result.commandsRun, 0);
    assert.equal(result.packages.length, 6);
  }
  assert.equal(f.calls.length, 0);
  assert.ok(!fs.existsSync(path.join(f.root, 'website/build-farm')));
});

test('incomplete release fails before any command or output', async t => {
  const f = fixture(t);
  f.hooks.getMetadata = async name => { throw new Error('[RELEASE_NOT_PUBLISHED] ' + name); };
  await assert.rejects(buildPublishedDocs(f.options, f.hooks), /RELEASE_NOT_PUBLISHED/);
  assert.equal(f.calls.length, 0);
});

test('main deployment before npm publication fails without commands or replacing existing output', async t => {
  const f = fixture(t);
  f.put('website/build-farm/index.html', 'previous successful production output');
  f.hooks.getMetadata = async name => { throw new Error('[RELEASE_NOT_PUBLISHED] ' + name); };
  for (const phase of ['install', 'build']) {
    await assert.rejects(buildPublishedDocs({ ...f.options, phase, environment: { VERCEL_ENV: 'production', VERCEL_GIT_COMMIT_REF: 'main' } }, f.hooks), /RELEASE_NOT_PUBLISHED/);
    assert.equal(f.calls.length, 0);
    assert.equal(fs.readFileSync(path.join(f.root, 'website/build-farm/index.html'), 'utf8'), 'previous successful production output');
    assert.deepEqual(fs.readdirSync(path.join(f.root, 'website')).filter(name => name.startsWith('build-farm')), ['build-farm']);
  }
});

test('rejects registry cohort drift, non-registry inputs, missing integrity and install scripts', async t => {
  const f = fixture(t);
  const plan = deploymentPlan(f.website, f.manifests, target);
  for (const mutate of [
    meta => { meta.version = '1.7.11'; },
    meta => { meta.dist.integrity = ''; },
    meta => { meta.dist.tarball = 'https://example.com/package.tgz'; },
    meta => { meta.dependencies['@farmfe/core'] = 'workspace:*'; },
    meta => { meta.dependencies['@farmfe/core'] = '1.7.11'; },
    meta => { delete meta.dependencies['@farmfe/core']; },
    meta => { meta.scripts = { install: 'cargo build' }; }
  ]) {
    const metadata = clone(f.metadata.get('@farmfe/docs')); mutate(metadata);
    await assert.rejects(preflight(plan, async name => name === '@farmfe/docs' ? metadata : clone(f.metadata.get(name))), /REGISTRY|COHORT|Published|INSTALL_SCRIPT/);
  }
});

test('third-party registry prereleases and semver ranges are accepted without relaxing Farm pins', async t => {
  const f = fixture(t);
  for (const selector of ['1.0.0-next.25', '^1.0.0-next.25', '>=1.0.0-next.25 <2.0.0', '~1.0.0-next.25+build.7', '1.0.0-next.25 || ^2.0.0', '1.0.0-next.25 - 2.0.0', '*', '18.x', '>= 18', '^0.7.0']) {
    f.website.dependencies['@polka/compression'] = selector;
    f.metadata.get('@farmfe/core').dependencies = { '@polka/compression': selector };
    const plan = deploymentPlan(f.website, f.manifests, target);
    assert.equal(plan.manifest.dependencies['@polka/compression'], selector);
    await preflight(plan, f.hooks.getMetadata);
  }
  f.metadata.get('@farmfe/docs').dependencies['@farmfe/core'] = '2.0.0-next.25';
  await assert.rejects(preflight(deploymentPlan(f.website, f.manifests, target), f.hooks.getMetadata), /COHORT_MISMATCH/);
});

test('third-party selectors reject source protocols, aliases, tags and malformed prereleases', async t => {
  const f = fixture(t);
  const plan = deploymentPlan(f.website, f.manifests, target);
  for (const selector of ['file:../pkg', 'link:../pkg', 'workspace:*', 'npm:other@1.0.0', 'git+https://example.com/pkg.git', 'git@github.com:org/repo.git', 'github:org/repo', 'https://example.com/pkg.tgz', '../pkg', '/pkg', 'latest', 'next', '1.0.0-01', '1.0-next', '1.0.0-next/other', '1.0.0-next:other']) {
    f.website.dependencies.react = selector;
    assert.throws(() => deploymentPlan(f.website, f.manifests, target), /REGISTRY_ONLY/);
    f.metadata.get('@farmfe/core').dependencies = { react: selector };
    await assert.rejects(preflight(plan, f.hooks.getMetadata), /REGISTRY_ONLY/);
  }
  assert.equal(f.calls.length, 0);
});

test('requires target native wrapper and stable exact release versions', t => {
  const f = fixture(t);
  assert.throws(() => deploymentPlan(f.website, f.manifests, 'darwin-arm64'), /NATIVE_TARGET_MISSING/);
  f.manifests.get('@farmfe/core').version = '2.0.0-beta.1';
  assert.throws(() => deploymentPlan(f.website, f.manifests, target), /EXACT_RELEASE_REQUIRED/);
  f.website.dependencies.react = 'file:../react';
  f.manifests.get('@farmfe/core').version = '2.0.0';
  assert.throws(() => deploymentPlan(f.website, f.manifests, target), /REGISTRY_ONLY/);
});

test('synthetic successful build is isolated, frozen, scripts-disabled and copies verified output', async t => {
  const f = fixture(t);
  f.put('website/node_modules/private.txt', 'must not copy');
  f.put('website/.env.local', 'must not copy');
  f.put('website/build-farm/old.html', 'previous successful output');
  const execute = f.hooks.execute;
  f.hooks.execute = async (...args) => {
    const cwd = args[2].cwd;
    assert.ok(!cwd.startsWith(f.root + path.sep));
    assert.ok(!fs.existsSync(path.join(cwd, '.env.local')));
    assert.ok(!fs.existsSync(path.join(cwd, 'node_modules/private.txt')));
    assert.ok(!fs.existsSync(path.join(cwd, 'build-farm/old.html')));
    assert.equal(read(path.join(cwd, 'package.json')).dependencies['@farmfe/core-linux-x64-gnu'], '2.0.0');
    if (args[0] === process.execPath) {
      // Resolve both paths while the temporary consumer still exists. On Windows,
      // native real paths can use a different spelling from the temporary cwd.
      const canonical = file => {
        const resolved = path.normalize(fs.realpathSync.native(file));
        return process.platform === 'win32' ? resolved.toLowerCase() : resolved;
      };
      const expected = path.join(cwd, 'node_modules/@farmfe/cli/bin/farm.mjs');
      const canonicalCwd = canonical(cwd);
      const canonicalCli = canonical(args[1][0]);
      assert.ok(isInside(canonicalCwd, canonicalCli), 'CLI must resolve inside the consumer');
      assert.equal(canonicalCli, canonical(expected), 'CLI must be the installed consumer executable');
    }
    return execute(...args);
  };
  const receipt = await buildPublishedDocs(f.options, f.hooks);
  assert.equal(receipt.commandsRun, 4);
  assert.equal(receipt.nativePackages.length, 2);
  assert.equal(f.calls[0].command, 'corepack');
  assert.ok(f.calls.slice(0, 2).every(call => call.args.includes('--ignore-scripts') && call.args.includes('--ignore-workspace')));
  assert.ok(f.calls[1].args.includes('--frozen-lockfile'));
  assert.ok(f.calls.slice(2).every(call => call.command === process.execPath));
  assert.deepEqual(f.calls.slice(2).map(call => call.args.slice(1)), [['--help'], ['build', '--config', 'farm.config.ts']]);
  assert.equal(fs.readFileSync(path.join(f.root, 'website/build-farm/THIRD_PARTY_NOTICES.txt'), 'utf8'), 'original notice');
  assert.ok(fs.readdirSync(path.join(f.root, 'website')).some(name => name.startsWith('build-farm.previous-')));
  assert.ok(!fs.existsSync(f.calls[0].cwd));
});

test('failed install stops without fallback and retains previous output', async t => {
  const f = fixture(t);
  f.put('website/build-farm/index.html', 'keep previous');
  f.hooks.execute = async (...args) => { f.calls.push(args); throw new Error('synthetic install failure'); };
  await assert.rejects(buildPublishedDocs(f.options, f.hooks), /install failure/);
  assert.equal(f.calls.length, 1);
  assert.equal(fs.readFileSync(path.join(f.root, 'website/build-farm/index.html'), 'utf8'), 'keep previous');
  assert.ok(!fs.existsSync(f.calls[0][2].cwd));
});

test('lock scanner distinguishes registry package keys from source values', () => {
  assert.doesNotThrow(() => assertRegistryLock(`
lockfileVersion: '9.0'
snapshots:
  postcss-pseudo-class-any-link@10.0.1:
    dependencies:
      postcss-pseudo-class-any-link: 10.0.1(postcss@8.5.32)
      cp-file: 10.0.0
packages:
  '@farmfe/core@2.0.0':
    engines: {node: '>=10.14', npm: '>=6', yarn: '>=1'}
    resolution: {integrity: sha512-abcdef, tarball: 'https://registry.npmjs.org/@farmfe/core/-/core-2.0.0.tgz'}
`));
  for (const value of ['workspace:*', 'link:../pkg', 'file:../pkg', 'portal:../pkg', 'patch:pkg', 'npm:other@1.0.0', 'git+https://example.com/repo.git', 'git://example.com/repo', 'git@github.com:org/repo', 'github:org/repo', 'gitlab:org/repo', 'bitbucket:org/repo', 'ssh://example.com/repo']) {
    for (const encoded of [value, `'${value}'`, JSON.stringify(value), JSON.stringify(value).replace(':', '\\u003a')]) {
      assert.throws(() => assertRegistryLock(`importers:
  .:
    dependencies:
      pkg:
        specifier: ${encoded}
`), /REGISTRY_ONLY/, encoded);
      assert.throws(() => assertRegistryLock(`packages:
  'pkg@${value}':
    resolution: {}
`), /REGISTRY_ONLY/, value);
    }
  }
  for (const text of [
    'resolution: {tarball: "https://example.com/pkg.tgz"}',
    'resolution: {tarball: "https://registry.npmjs.org.evil.test/pkg.tgz"}',
    'resolution: {tarball: "https://user:secret@registry.npmjs.org/pkg.tgz"}',
    String.raw`specifier: "l\u0069nk:../pkg"`,
    String.raw`specifier: "file\x3a../pkg"`,
    'specifier: "file:"',
    'specifier: "npm: other@1.0.0"',
    'specifier: &source link:../pkg',
    'specifier: *source',
    'specifier: !!str "file:../pkg"',
    `specifier: >-
  link:../pkg`,
    String.raw`specifier: "li\
  nk:../pkg"`
  ]) assert.throws(() => assertRegistryLock(text), /REGISTRY_ONLY/, text);
});

test('lock resolutions reject non-registry fetchers and ambiguous full tarball scalars', () => {
  const prefix = 'resolution: {integrity: sha512-abcdef, tarball: ';
  for (const url of [
    'https://evil.test/pkg.tgz', 'https:/evil.test/pkg.tgz', 'https:evil.test/pkg.tgz',
    String.raw`https:\evil.test/pkg.tgz`, String.raw`https:\\evil.test/pkg.tgz`,
    '//evil.test/pkg.tgz', String.raw`\\evil.test\pkg.tgz`,
    'ftp://evil.test/pkg.tgz', '/tmp/pkg.tgz', '../outside.tgz',
    'https://registry.npmjs.org.evil.test/pkg.tgz',
    'https://registry.npmjs.org@evil.test/pkg.tgz',
    'https://registry.npmjs.org}@evil.test/pkg.tgz',
    'https://registry.npmjs.org,@evil.test/pkg.tgz',
    'https://registry.npmjs.org]@evil.test/pkg.tgz',
    'https://registry.npmjs.org\n@evil.test/pkg.tgz',
    'https://registry.npmjs.org\t@evil.test/pkg.tgz',
    String.raw`https://registry.npmjs.org\@evil.test/pkg.tgz`
  ]) {
    for (const scalar of [JSON.stringify(url), `'${url}'`, url]) assert.throws(() => assertRegistryLock(prefix + scalar + '}'), /REGISTRY_ONLY/, scalar);
  }
  for (const resolution of [
    '{directory: /tmp/pkg, type: directory}', '{type: directory, directory: ../outside}',
    '{integrity: sha512-abcdef, type: tarball}', '{type: git, repo: github.com/org/repo}',
    '{integrity: sha512-abcdef, commit: 1234}', '{integrity: sha512-abcdef, directory: ../outside}',
    '{tarball: https://registry.npmjs.org/pkg/-/pkg-1.0.0.tgz}',
    '{integrity: sha512-abcdef, integrity: sha512-other}',
    '{integrity: sha512-abcdef, "tarball": "https://evil.test/pkg.tgz"}',
    `
  type: directory
  directory: ../outside`
  ]) assert.throws(() => assertRegistryLock('resolution: ' + resolution), /REGISTRY_ONLY/, resolution);
  for (const text of [
    '"resolution": {directory: ../outside}',
    'resolution : {directory: ../outside}',
    'packages: {pkg: {resolution: {directory: ../outside, type: directory}}}'
  ]) assert.throws(() => assertRegistryLock(text), /REGISTRY_ONLY/, text);
  assert.doesNotThrow(() => assertRegistryLock(prefix + '"https://registry.npmjs.org/@farmfe/core/-/core-2.0.0.tgz"}'));
});

test('lock scanner rejects explicit mapping keys and raw or decoded control characters', () => {
  for (const text of [
    '? resolution\n: {type: directory, directory: ../outside}',
    '? "resolution"\n: {type: directory, directory: ../outside}',
    "? 'resolution'\n: {type: directory, directory: ../outside}",
    '? |-\n  resolution\n: {type: directory, directory: ../outside}',
    '{? resolution: {type: directory, directory: ../outside}}',
    '{? "resolution": {type: directory, directory: ../outside}}',
    '{?"resolution":{type: directory, directory: ../outside}}',
    '?|-\n  resolution\n:{type: directory, directory: ../outside}',
    ': {type: directory, directory: ../outside}',
    '{: {type: directory, directory: ../outside}}'
  ]) assert.throws(() => assertRegistryLock(text), /REGISTRY_ONLY/, text);
  for (const code of [...Array.from({length: 32}, (_, i) => i), ...Array.from({length: 33}, (_, i) => i + 127)]) {
    const character = String.fromCharCode(code);
    assert.throws(() => assertRegistryLock('specifier: ' + JSON.stringify('1.0.0' + character)), /REGISTRY_ONLY/, String(code));
    if (code !== 10 && code !== 13) assert.throws(() => assertRegistryLock('specifier: 1.0.0' + character), /REGISTRY_ONLY/, String(code));
  }
  assert.throws(() => assertRegistryLock('specifier: 1.0.0\rmalformed'), /REGISTRY_ONLY/);
  assert.doesNotThrow(() => assertRegistryLock("lockfileVersion: '9.0'\r\npackages: {}\r\n"));
});

test('lock scanner rejects collection-valued mapping keys without rejecting array values', () => {
  for (const key of ['[resolution]', '["resolution"]', "['resolution']", '[[resolution]]', '{resolution: null}', '[\n  resolution\n]']) {
    for (const spacing of ['', ' ']) assert.throws(() => assertRegistryLock(`${key}${spacing}: {type: directory, directory: ../outside}`), /REGISTRY_ONLY/, key);
  }
  assert.doesNotThrow(() => assertRegistryLock(`
packages:
  '@farmfe/core-darwin-arm64@2.0.0':
    resolution: {integrity: sha512-abcdef}
    cpu: [arm64]
    os: ['darwin']
    libc: ["glibc"]
    bundledDependencies: []
`));
});

test('lock sources, integrity drift and unexpected local native payload fail before CLI', async t => {
  for (const problem of ['workspace', 'integrity', 'local native']) {
    const f = fixture(t), execute = f.hooks.execute;
    f.hooks.execute = async (...args) => {
      await execute(...args);
      const cwd = args[2].cwd;
      if (args[1].includes('--lockfile-only') && problem !== 'local native') fs.writeFileSync(path.join(cwd, 'pnpm-lock.yaml'), problem === 'workspace' ? 'workspace:*' : 'no integrity');
      if (args[1].includes('--frozen-lockfile') && problem === 'local native') fs.writeFileSync(path.join(cwd, 'node_modules/@farmfe/core/binding/local.node'), 'wrong native');
    };
    await assert.rejects(buildPublishedDocs(f.options, f.hooks), /REGISTRY_ONLY|LOCK_INTEGRITY|UNEXPECTED_LOCAL_NATIVE/);
    assert.ok(f.calls.length <= 2);
  }
});

test('native override and injected loader are rejected before metadata or commands', async t => {
  const f = fixture(t);
  for (const environment of [{ NODE_PATH: '/elsewhere' }, { NAPI_RS_NATIVE_LIBRARY_PATH: '/local.node' }, { NODE_OPTIONS: '--import=/hook.mjs' }]) await assert.rejects(buildPublishedDocs({ ...f.options, environment }, f.hooks), /BINDING_OVERRIDE_FORBIDDEN/);
  assert.equal(f.calls.length, 0);
});

test('Vercel enables main Git deployments with unchanged registry-only commands', () => {
  const config = read(path.join(repository, 'website/vercel.json'));
  assert.equal(config.installCommand, 'node ../scripts/farm-docs-vercel-build.mjs --install');
  assert.equal(config.buildCommand, 'node ../scripts/farm-docs-vercel-build.mjs');
  assert.equal(config.outputDirectory, 'build-farm');
  assert.deepEqual(config.git.deploymentEnabled, { main: true });
});

test('actual CLI rejects unknown options without any build', () => {
  const result = spawnSync(process.execPath, [fileURLToPath(new URL('./farm-docs-vercel-build.mjs', import.meta.url)), '--source-fallback'], { encoding: 'utf8' });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /ARGUMENTS/);
});

test('Quick Start describes published v2 without changing feature scope or claiming pinned creation commands', () => {
  const en = fs.readFileSync(path.join(repository, 'website/docs/quick-start.mdx'), 'utf8');
  const zh = fs.readFileSync(path.join(repository, 'website/i18n/zh/docusaurus-plugin-content-docs/current/quick-start.mdx'), 'utf8');
  assert.match(en, /Farm v2 is available on npm/);
  assert.match(en, /rather than pinning an exact version/);
  assert.match(en, /official Rust plugin overview/);
  assert.doesNotMatch(en, /v2 candidate documentation|until the required official packages are published/);
  assert.match(zh, /Farm v2 已发布到 npm/);
  assert.match(zh, /而非固定的精确版本/);
  assert.match(zh, /官方 Rust 插件总览/);
  assert.doesNotMatch(zh, /这是 v2 候选文档|所需官方包发布前/);
});
