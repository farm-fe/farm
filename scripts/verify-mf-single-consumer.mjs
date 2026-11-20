// Credential-free candidate install. Nothing is published or granted.
// cspell:words USERCONFIG GLOBALCONFIG
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { hostNativeTarget } from '../rust-plugins/module-federation/scripts/native-platform.mjs';
import { owner, version, readMfBundleForTest } from './prepare-mf-single-package.mjs';
import { resolveNpmCli } from './v2-npm-cli.mjs';
import { sha256 } from './release-archive.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const artifacts = path.resolve(process.argv[2]);
const receipt = readMfBundleForTest(root, artifacts);
const { abi } = hostNativeTarget();
const member = receipt.native.find(row => row.abi === abi);
assert.ok(member, 'Missing exact native candidate platform');
const scratch = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'farm-mf-bundle-consumer-')));
try {
  const npm = resolveNpmCli();
  const env = Object.fromEntries(Object.entries(process.env).filter(([key]) =>
    !/TOKEN|NPM_CONFIG|FARM_LIB_CORE_PATH|NAPI_RS_NATIVE_LIBRARY_PATH|NODE_PATH|NODE_OPTIONS/i.test(key)));
  env.NPM_CONFIG_USERCONFIG = path.join(scratch, 'empty-user.npmrc');
  env.NPM_CONFIG_GLOBALCONFIG = path.join(scratch, 'empty-global.npmrc');
  for (const name of ['empty-user.npmrc', 'empty-global.npmrc']) fs.writeFileSync(path.join(scratch, name), '');
  fs.writeFileSync(path.join(scratch, 'package.json'), JSON.stringify({ name: 'mf-bundle-consumer', private: true, type: 'module' }));
  execFileSync(process.execPath, [npm, 'install', '--ignore-scripts', '--no-audit', '--no-fund',
    '--registry=https://registry.npmjs.org', '--cache', path.join(scratch, '.cache'),
    '@farmfe/core@2.0.2', path.join(artifacts, receipt.file)], { cwd: scratch, env, stdio: 'inherit', timeout: 180000 });
  const lock = JSON.parse(fs.readFileSync(path.join(scratch, 'package-lock.json')));
  assert.equal(lock.packages['node_modules/@farmfe/core']?.version, '2.0.2');
  assert.equal(lock.packages['node_modules/' + owner]?.version, version);
  assert.ok(Object.values(lock.packages).every(pkg => !pkg.link), 'Linked packages are not independent evidence');
  const binary = path.join(scratch, 'node_modules', owner, 'native', abi, 'index.farm');
  assert.equal(sha256(fs.readFileSync(binary)), member.binarySha256);
  fs.writeFileSync(path.join(scratch, 'value.js'), 'export const value = 42;\n');
  fs.writeFileSync(path.join(scratch, 'probe.mjs'), `import assert from 'node:assert/strict';
import fs from 'node:fs';
import federation from ${JSON.stringify(owner)};
import { build } from '@farmfe/core';
const plugin = federation({name:'farmInstalledRemote', exposes:{'./Value':'./value.js'}});
assert.equal(fs.realpathSync(plugin[0]), fs.realpathSync(${JSON.stringify(binary)}));
await build({root:process.cwd(),mode:'production',configFile:false,plugins:[plugin],
  compilation:{input:{remoteEntry:'farm-mf-native:container'},output:{path:'dist',targetEnv:'browser-esnext'},
    persistentCache:false,lazyCompilation:false}});
assert.ok(fs.statSync('dist/remoteEntry.js').isFile());
`);
  execFileSync(process.execPath, ['probe.mjs'], { cwd: scratch, env, stdio: 'inherit', timeout: 120000 });
  console.log(JSON.stringify({ status: 'passed', abi, version, core: '2.0.2', archiveSha256: receipt.sha256, published: false }));
} finally {
  fs.rmSync(scratch, { recursive: true, force: true });
}
