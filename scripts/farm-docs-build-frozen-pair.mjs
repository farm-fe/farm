import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const node = process.execPath;
const HOST = 'registry.npmjs.org';
const VERSION = '2.0.0';
const PAIR = ['docs-theme-default', 'docs'];
const RUST = {
  docs: '0.0.1', 'docs-seo': '0.0.1', 'docs-snapshot': '0.0.1', 'docs-snippets': '0.0.1',
  'ssr-css': '0.0.1', 'syntax-highlight': '0.0.1', react: '2.0.0', 'replace-dirname': '2.0.0'
};
const HEX = /^[a-f0-9]{40}$/;
const LOCK_SHA256 = '4bee011563115524406c15a541463d9d2077ce5b8e10ba9951c87ba4ff90cdc2';
const MANIFEST_SHA256 = 'b487628b21358b62887458b049ba2314e32945c16cad6014155f7008a76f422d';
const read = file => JSON.parse(fs.readFileSync(file));
const hash = (value, algorithm = 'sha256', encoding = 'hex') => createHash(algorithm).update(value).digest(encoding);
const paths = {
  docs: ['client.html', ...['input','page','plan','public','sidebar'].map(name => 'schema/' + name + '.json')],
  'docs-theme-default': ['src/style.css']
};
function write(file, value) {
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + '\n', { flag: 'wx' });
}
function original(root, name) { return read(path.join(root, 'packages', name, 'package.json')); }
function run(command, args, cwd, env, input) {
  return execFileSync(command, args, { cwd, env, input, maxBuffer: 24 * 1024 * 1024, timeout: 180_000, stdio: ['pipe', 'pipe', 'inherit'] });
}
function publicTarball(row) {
  assert.equal(typeof row.resolved, 'string');
  const url = new URL(row.resolved);
  assert.equal(url.origin, 'https://' + HOST);
  assert(!url.username && !url.password && !url.search && !url.hash && url.pathname.endsWith('.tgz'));
  assert.match(row.integrity, /^sha512-[A-Za-z0-9+/]{86}==$/);
  assert(!row.link, 'Public dependency lock contains a link');
}
function localArchiveBytes(file) {
  const meta = fs.lstatSync(file);
  assert(meta.isFile() && meta.size > 0 && meta.size < 32 * 1024 * 1024);
  const body = fs.readFileSync(file);
  assert.equal(body.length, meta.size);
  return body;
}

const [repositoryArg, sourceSHA, sourceTree, outputArg, fixtureArg, npmArg, pnpmArg, ...extra] = process.argv.slice(2);
assert(repositoryArg && outputArg && fixtureArg && npmArg && pnpmArg && extra.length === 0,
  'Usage: node build.mjs REPO SOURCE_SHA SOURCE_TREE NEW_OUTPUT FIXTURE_DIR NPM_CLI_JS PNPM_CLI_CJS');
assert(HEX.test(sourceSHA) && HEX.test(sourceTree), 'Independent exact commit and tree required');
assert.equal(process.version, 'v22.22.2');
assert.equal(process.platform, 'linux'); assert.equal(process.arch, 'x64');
assert(process.report.getReport().header.glibcVersionRuntime, 'Linux x64 glibc required');
const repository = fs.realpathSync(repositoryArg), fixture = fs.realpathSync(fixtureArg);
const output = path.resolve(outputArg), outputParent = fs.realpathSync(path.dirname(output));
assert.equal(outputParent, path.dirname(output), 'Output parent must be a canonical non-symlink path');
assert(output !== repository && !output.startsWith(repository + '/'), 'Output must be outside the checkout');
const npm = fs.realpathSync(npmArg), pnpm = fs.realpathSync(pnpmArg);
const gitEnv = { PATH: '/usr/bin:/bin', GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: '/dev/null', GIT_TERMINAL_PROMPT: '0', LC_ALL: 'C' };
const git = args => run('git', args, repository, gitEnv);
assert.equal(git(['rev-parse', '--verify', sourceSHA + '^{commit}']).toString().trim(), sourceSHA);
assert.equal(git(['rev-parse', '--verify', sourceSHA + '^{tree}']).toString().trim(), sourceTree, 'Exact Git tree differs');
const pkgFile = path.join(fixture, 'package.json'), lockFile = path.join(fixture, 'package-lock.json');
assert.equal(hash(fs.readFileSync(pkgFile)), MANIFEST_SHA256, 'Public fixture manifest changed');
assert.equal(hash(fs.readFileSync(lockFile)), LOCK_SHA256, 'Public fixture lock changed');
const fixtureManifest = read(pkgFile), lock = read(lockFile);
assert.equal(lock.lockfileVersion, 3);
assert.deepEqual(fixtureManifest.dependencies, lock.packages[''].dependencies);
for (const [name, row] of Object.entries(lock.packages)) if (name) publicTarball(row);
fs.mkdirSync(output, { mode: 0o700 }); // existing directory fails, never overwrite a receipt
const scratch = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-docs-frozen-pair-'));
const src = path.join(scratch, 'source'), dependencies = path.join(scratch, 'dependencies'), home = path.join(scratch, 'home');
for (const directory of [src, dependencies, home]) fs.mkdirSync(directory);
try {
  fs.writeFileSync(path.join(home, 'user.npmrc'), ''); fs.writeFileSync(path.join(home, 'global.npmrc'), '');
  const env = {
    PATH: path.dirname(node) + ':/usr/bin:/bin', HOME: home, LC_ALL: 'C.UTF-8', TZ: 'UTC', CI: 'true',
    npm_config_userconfig: path.join(home, 'user.npmrc'), npm_config_globalconfig: path.join(home, 'global.npmrc'),
    npm_config_registry: 'https://' + HOST + '/', npm_config_ignore_scripts: 'true', npm_config_audit: 'false',
    npm_config_fund: 'false', npm_config_fetch_retries: '0'
  };
  // Local development proxies are allowed only without embedded credentials.
  for (const field of ['HTTPS_PROXY','HTTP_PROXY']) if (process.env[field]) {
    const proxy = new URL(process.env[field]);
    assert(!proxy.username && !proxy.password && ['http:','https:'].includes(proxy.protocol));
    env[field] = process.env[field];
  }
  if (process.env.NODE_EXTRA_CA_CERTS) env.NODE_EXTRA_CA_CERTS = process.env.NODE_EXTRA_CA_CERTS;
  const nod = (args, cwd) => run(node, args, cwd, env);
  assert.equal(nod([npm, '--version'], scratch).toString().trim(), '11.19.1');
  assert.equal(nod([pnpm, '--version'], scratch).toString().trim(), '9.4.0');
  const gitPaths = ['tsconfig.base.json', ...PAIR.map(name => 'packages/' + name),
    ...Object.keys(RUST).map(name => 'rust-plugins/' + name + '/package.json'),
    'scripts/farm-docs-package-contract.mjs', 'scripts/v2-normal-pack-contract.mjs', 'scripts/farm-docs-extract-package.py'];
  const checked = git(['ls-tree', '-rz', sourceSHA, '--', ...gitPaths]).toString().split('\0').filter(Boolean);
  assert(checked.length > 20 && checked.every(entry => /^100(?:644|755) blob [a-f0-9]{40}\t/.test(entry)), 'Unexpected source Git blob mode');
  run('tar', ['-xf', '-'], src, gitEnv, git(['archive','--format=tar',sourceSHA,'--',...gitPaths]));
  for (const name of PAIR) {
    const p = original(src, name);
    assert.equal(p.name, '@farmfe/' + name); assert.equal(p.version, VERSION);
  }
  assert.equal(original(src, 'docs')['dependencies']['@farmfe/docs-theme-default'], 'workspace:*');
  assert.equal(original(src, 'docs-theme-default')['peerDependencies']['@farmfe/docs'], VERSION);
  const versions = new Map(PAIR.map(name => ['@farmfe/' + name, VERSION]));
  for (const [name, expected] of Object.entries(RUST)) {
    const p = read(path.join(src, 'rust-plugins', name, 'package.json'));
    assert.equal(p.name, '@farmfe/plugin-' + name); assert.equal(p.version, expected);
    versions.set(p.name, expected);
  }
  fs.writeFileSync(path.join(src,'pnpm-workspace.yaml'), "packages:\n  - 'packages/*'\n  - 'rust-plugins/*'\n");
  write(path.join(src,'package.json'), { name: 'farm-docs-pair-source', private: true });
  fs.copyFileSync(pkgFile, path.join(dependencies, 'package.json'));
  fs.copyFileSync(lockFile, path.join(dependencies, 'package-lock.json'));
  nod([npm, 'ci', '--ignore-scripts', '--no-audit', '--no-fund'], dependencies);
  for (const [name, version] of Object.entries(fixtureManifest.dependencies)) {
    const actual = read(path.join(dependencies, 'node_modules', name, 'package.json'));
    assert.equal(actual.name, name); assert.equal(actual.version, version);
  }
  for (const name of ['core', ...Object.keys(RUST).map(n => 'plugin-' + n)]) {
    const n = '@farmfe/' + name + '-linux-x64-gnu';
    const row = lock.packages['node_modules/' + n];
    assert(row, 'Linux native package is missing from locked closure: ' + n);
    const d = path.join(dependencies, 'node_modules', n);
    const installed = read(path.join(d, 'package.json'));
    assert.equal(installed.version, row.version);
    const native = fs.readdirSync(d).filter(f => /\.(node|farm)$/.test(f));
    assert.equal(native.length, 1);
    const header = fs.readFileSync(path.join(d, native[0]));
    assert(header.subarray(0, 5).equals(Buffer.from([0x7f,0x45,0x4c,0x46,2])) && header.readUInt16LE(18) === 62,
      'Expected one x64 ELF Linux payload: ' + n);
  }
  fs.symlinkSync(path.join(dependencies, 'node_modules'), path.join(src, 'node_modules'));
  for (const name of PAIR) {
    fs.symlinkSync(path.join(src,'packages',name),path.join(dependencies,'node_modules','@farmfe',name));
    // pnpm 9 resolves workspace:* at the package's own node_modules when packing.
    fs.symlinkSync(path.join(dependencies, 'node_modules'),path.join(src,'packages',name,'node_modules'));
  }
  const tsc = path.join(dependencies,'node_modules','typescript','bin','tsc');
  for (const name of [...PAIR].reverse()) nod([tsc,'-p','tsconfig.json'],path.join(src,'packages',name));
  const verifier = await import(pathToFileURL(path.join(src,'scripts','farm-docs-package-contract.mjs')));
  const packages = [];
  for (const name of PAIR) {
    const before = new Set(fs.readdirSync(output));
    nod([pnpm,'pack','--pack-destination',output],path.join(src,'packages',name));
    const additions = fs.readdirSync(output).filter(file => !before.has(file));
    assert.deepEqual(additions, ['farmfe-' + name + '-' + VERSION + '.tgz']);
    const archive = path.join(output, additions[0]), bytes = localArchiveBytes(archive), archiveSha256 = hash(bytes);
    const extracted = path.join(scratch,'extract-' + name);
    run('python3',[path.join(src,'scripts','farm-docs-extract-package.py'),
      '--archive',archive,'--sha256',archiveSha256,'--destination',extracted,'--repository',src,
      '--receipt',path.join(scratch,'extract-' + name + '.json')],scratch,env);
    const verified = await verifier.verifyDocsPackageDirectory(extracted, {
      original:original(src,name), versions, requiredFiles:paths[name], forbiddenRoots:[src,repository,scratch]
    });
    assert.equal(verified.name, '@farmfe/' + name);
    packages.push({name:verified.name,version:VERSION,archiveSha256,archiveBytes:bytes.length,
      integrity:'sha512-' + hash(bytes,'sha512','base64')});
  }
  write(path.join(output,'frozen-pair.json'),{schemaVersion:1,kind:'docs-frozen-pair-v1',sourceSHA,sourceTree,packages});
} finally {
  fs.rmSync(scratch,{recursive:true,force:true});
}
