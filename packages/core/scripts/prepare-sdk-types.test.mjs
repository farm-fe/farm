import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { mkdtemp, mkdir, readFile, readdir, writeFile, symlink, realpath, access, cp, unlink } from 'node:fs/promises';
import { createRequire } from 'node:module';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { test } from 'node:test';
import ts from 'typescript';
import { prepareSdkTypes, upstream } from './prepare-sdk-types.mjs';

const packageRoot = fileURLToPath(new URL('..', import.meta.url));
const sourceCore = path.resolve(process.env.FARM_SDK_TYPES_TEST_SOURCE_CORE ?? packageRoot);
const repository = path.resolve(process.env.FARM_SDK_TYPES_TEST_REPOSITORY ?? path.join(sourceCore, '../..'));
const hash = (bytes) => createHash('sha256').update(bytes).digest('hex');
async function put(file, bytes) { await mkdir(path.dirname(file), { recursive: true }); await writeFile(file, bytes); }
async function link(file, target) { await mkdir(path.dirname(file), { recursive: true }); await symlink(await realpath(target), file, 'junction'); }
async function emit(relative, target) {
  const file = path.join(repository, relative);
  const program = ts.createProgram([file], { noCheck: true, declaration: true, emitDeclarationOnly: true,
    target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.NodeNext, moduleResolution: ts.ModuleResolutionKind.NodeNext });
  let output;
  const result = program.emit(program.getSourceFile(file), (name, text) => { if (name.endsWith('.d.ts')) output = text; });
  assert.equal(result.emitSkipped, false); assert.ok(output, relative);
  await put(target, output);
}

test('normal prepack prepares real Core CJS types and the installed package passes strict NodeNext without fictitious APIs', async (t) => {
  assert.ok(os.freemem() >= 1.5 * 1024 ** 3, 'Run this single TypeScript test only with sufficient memory');
  const temporary = await mkdtemp(path.join(process.env.FARM_SDK_TYPES_TEST_TMP ?? os.tmpdir(), 'farm-sdk-types-'));
  const core = path.join(temporary, 'core');
  await put(path.join(core, 'package.json'), await readFile(path.join(packageRoot, 'package.json')));
  await put(path.join(core, 'THIRD_PARTY_NOTICES.md'), await readFile(path.join(packageRoot, 'THIRD_PARTY_NOTICES.md')));
  await put(path.join(core, 'LICENSE'), await readFile(path.join(sourceCore, 'LICENSE')));
  // Use actual, normally built runtime/native artifacts. They are never
  // fabricated by this test. Normal prepack must run the existing real runtime
  // export-parity check; unpacked consumers below remain type-only.
  await access(path.join(sourceCore, 'dist/index.d.cts'));
  await cp(path.join(sourceCore, 'dist'), path.join(core, 'dist'), { recursive: true });
  await cp(path.join(sourceCore, 'binding'), path.join(core, 'binding'), { recursive: true });
  await cp(path.join(sourceCore, 'licenses'), path.join(core, 'licenses'), { recursive: true });
  await cp(path.join(sourceCore, 'types'), path.join(core, 'types'), { recursive: true });
  await put(path.join(core, 'client.d.ts'), await readFile(path.join(sourceCore, 'client.d.ts')));
  // Tests use existing installed packages only. The proposed manifest makes
  // anymatch explicit; no dependency install or registry access is performed.
  for (const entry of await readdir(path.join(sourceCore, 'node_modules'), { withFileTypes: true })) {
    if (!['.bin', 'anymatch', 'typescript'].includes(entry.name) && (entry.isDirectory() || entry.isSymbolicLink()))
      await link(path.join(core, 'node_modules', entry.name), path.join(sourceCore, 'node_modules', entry.name));
  }
  const sourceRequire = createRequire(path.join(sourceCore, 'package.json'));
  const chokidarPackage = sourceRequire.resolve('chokidar/package.json');
  const chokidarRoot = path.dirname(chokidarPackage);
  const chokidarRequire = createRequire(chokidarPackage);
  await link(path.join(core, 'node_modules/anymatch'), path.dirname(chokidarRequire.resolve('anymatch/package.json')));
  const testRequire = createRequire(import.meta.url);
  await link(path.join(core, 'node_modules/typescript'), path.dirname(testRequire.resolve('typescript/package.json')));
  await put(path.join(core, 'scripts/prepare-sdk-types.mjs'), await readFile(new URL('./prepare-sdk-types.mjs', import.meta.url)));
  await put(path.join(core, 'scripts/prepare-cjs-types.mjs'), await readFile(path.join(sourceCore, 'scripts/prepare-cjs-types.mjs')));
  const original = await readFile(path.join(chokidarRoot, 'types/index.d.ts'));
  const license = await readFile(path.join(chokidarRoot, 'LICENSE'));
  const first = await prepareSdkTypes(core);
  assert.equal(first.imports.length, 3);
  const second = await prepareSdkTypes(core);
  assert.deepEqual(second.changed, [], 'repeat generation must preserve exact bytes and mtimes');
  const local = await readFile(path.join(core, 'dist/vendor/chokidar.d.ts'), 'utf8');
  assert.equal(local, original.toString('utf8').replace(' implements fs.FSWatcher', ''));
  assert.equal(hash(original), upstream.declarationSha256);
  assert.equal(hash(license), upstream.licenseSha256);
  assert.deepEqual(await readFile(path.join(core, 'licenses/chokidar-3.5.3-MIT.txt')), license);
  // Local normal pack: lifecycle scripts are enabled, registry access remains
  // offline. Removing only the staged CJS declaration proves prepack actually
  // regenerates it via authenticated runtime exports, rather than packing a
  // stale pre-existing file. No source checkout output is removed or changed.
  const npm = process.env.FARM_SDK_TYPES_NPM_CLI ?? path.join(path.dirname(process.execPath), 'node_modules/npm/bin/npm-cli.js');
  await access(npm);
  const tarballs = [];
  for (const iteration of [1, 2]) {
    const output = path.join(temporary, `pack-${iteration}`); await mkdir(output);
    await unlink(path.join(core, 'dist/index.d.cts'));
    const env = { ...process.env, npm_config_cache: path.join(temporary, 'npm-cache'), npm_config_audit: 'false',
      npm_config_fund: 'false', npm_config_ignore_scripts: 'false' };
    for (const key of ['NAPI_RS_NATIVE_LIBRARY_PATH', 'FARM_LIB_CORE_PATH']) delete env[key];
    const outputText = execFileSync(process.execPath, [npm, 'pack', '--offline', '--json', '--pack-destination', output],
      { cwd: core, encoding: 'utf8', env });
    const jsonStart = outputText.lastIndexOf('\n[');
    const result = JSON.parse(jsonStart >= 0 ? outputText.slice(jsonStart + 1) : outputText);
    await access(path.join(core, 'dist/index.d.cts'));
    for (const required of ['dist/index.d.cts', 'dist/vendor/chokidar.d.ts', 'dist/vendor/chokidar.provenance.json',
      'dist/vendor/chokidar-3.5.3.original.d.ts.txt', 'licenses/chokidar-3.5.3-MIT.txt']) {
      assert.ok(result[0].files.some((file) => file.path === required), `pack must include ${required}`);
    }
    tarballs.push(path.join(output, result[0].filename));
  }
  assert.equal(hash(await readFile(tarballs[0])), hash(await readFile(tarballs[1])), 'repeated local packs must match');
  const extracted = path.join(temporary, 'unpacked'); await mkdir(extracted);
  execFileSync('tar', ['-xzf', tarballs[0], '-C', extracted]);
  const packedCore = path.join(extracted, 'package');
  await link(path.join(packedCore, 'node_modules'), path.join(core, 'node_modules'));
  const consumer = path.join(temporary, 'consumer');
  await link(path.join(consumer, 'node_modules/@farmfe/core'), packedCore);
  // A real consumer has one selected Node ambient type version. Resolve the
  // repository's installed version explicitly instead of mixing private
  // transitive @types/node 18 and 22 trees via test-only dependency junctions.
  const nodeTypes = path.dirname(sourceRequire.resolve('@types/node/package.json'));
  await link(path.join(consumer, 'node_modules/@types/node'), nodeTypes);
  for (const pkg of ['qiankun', 'tailwindcss', 'babel', 'react-compiler']) {
    const target = path.join(consumer, 'node_modules/@farmfe/js-plugin-' + pkg);
    await put(path.join(target, 'package.json'), await readFile(path.join(repository, 'js-plugins', pkg, 'package.json')));
    const entry = pkg === 'qiankun' ? 'qiankun-farm-plugin' : 'index';
    const dist = pkg === 'qiankun' ? 'dist/plugin' : 'dist';
    await emit(`js-plugins/${pkg}/src/${entry}.ts`, path.join(target, dist, `${entry}.d.ts`));
    if (pkg === 'qiankun') {
      await put(path.join(target, dist, `${entry}.d.cts`), await readFile(path.join(target, dist, `${entry}.d.ts`)));
      await emit('js-plugins/qiankun/src/qiankun-farm-plugin-helper.ts', path.join(target, 'dist/helper/qiankun-farm-plugin-helper.d.ts'));
      await put(path.join(target, 'dist/helper/qiankun-farm-plugin-helper.d.cts'), await readFile(path.join(target, 'dist/helper/qiankun-farm-plugin-helper.d.ts')));
    }
    if (pkg === 'tailwindcss') await put(path.join(target, 'dist/index.d.cts'), await readFile(path.join(repository, 'js-plugins/tailwindcss/types/index.d.cts')));
    if (pkg === 'react-compiler') await emit('js-plugins/react-compiler/src/types.ts', path.join(target, 'dist/types.d.ts'));
  }
  const template = path.join(consumer, 'node_modules/fixture-plugin');
  const templateManifest = JSON.parse(await readFile(path.join(repository, 'packages/create-farm-plugin/templates/js/package.json'), 'utf8'));
  templateManifest.name = 'fixture-plugin'; await put(path.join(template, 'package.json'), JSON.stringify(templateManifest));
  await emit('packages/create-farm-plugin/templates/js/src/index.ts', path.join(template, 'build/esm/index.d.ts'));
  await put(path.join(template, 'build/cjs/index.d.cts'), `declare const plugin: typeof import('../esm/index.js', { with: { 'resolution-mode': 'import' } }).default;\nexport = plugin;\n`);
  for (const pkg of ['@babel/core', '@types/babel__core']) await link(path.join(consumer, 'node_modules', pkg), path.join(repository, 'js-plugins/babel/node_modules', pkg));
  await link(path.join(consumer, 'node_modules/@babel/types'), path.join(repository, 'js-plugins/react-compiler/node_modules/@babel/types'));
  const body = `q.qiankunFarmPlugin({appName:'fixture'});h.injectQiankun({bootstrap:async()=>{},mount:async()=>{},unmount:async()=>{}});
tailwind({filters:{moduleTypes:['css']}});plugin({});b.babel({priority:99,filters:{moduleTypes:['tsx']}});r.reactCompiler({compilerOptions:{}});
// @ts-expect-error required appName remains enforced
q.qiankunFarmPlugin({});
// @ts-expect-error moduleTypes remains string-only
tailwind({filters:{moduleTypes:[123]}});
// @ts-expect-error Babel ModuleType remains enforced
b.babel({priority:99,filters:{moduleTypes:[123]}});
// @ts-expect-error compilerOptions must remain an object
r.reactCompiler({compilerOptions:123});
`;
  const esm = `import * as q from '@farmfe/js-plugin-qiankun';import * as h from '@farmfe/js-plugin-qiankun/helper';import tailwind from '@farmfe/js-plugin-tailwindcss';import plugin from 'fixture-plugin';import * as b from '@farmfe/js-plugin-babel';import * as r from '@farmfe/js-plugin-react-compiler';\n`;
  const cjs = `import q=require('@farmfe/js-plugin-qiankun');import h=require('@farmfe/js-plugin-qiankun/helper');import tw=require('@farmfe/js-plugin-tailwindcss');import plugin=require('fixture-plugin');import b=require('@farmfe/js-plugin-babel');import r=require('@farmfe/js-plugin-react-compiler');const tailwind=tw.default;\n`;
  await put(path.join(consumer, 'consumer.mts'), esm + body);
  await put(path.join(consumer, 'consumer.cts'), cjs + body + `// @ts-expect-error template CJS export is callable, not a default namespace\nplugin.default({});\n`);
  await put(path.join(consumer, 'watcher.mts'), `import type {FSWatcher,WatchOptions} from './node_modules/@farmfe/core/dist/vendor/chokidar.js';
declare const watcher:FSWatcher;const options:WatchOptions={depth:2,ignored:[/ignore/,path=>path.includes('ignore')]};
watcher.add('src');watcher.unwatch('old');watcher.close();watcher.getWatched();watcher.on('change',(name,stats)=>{name.toUpperCase();stats?.isFile();});
// @ts-expect-error no fictitious native ref API
watcher.ref();
// @ts-expect-error no fictitious native unref API
watcher.unref();
// @ts-expect-error genuine option contract remains enforced
const bad:WatchOptions={depth:'deep'};
// @ts-expect-error genuine add contract remains enforced
watcher.add(123);void options;void bad;\n`);
  const program = ts.createProgram(['consumer.mts', 'consumer.cts', 'watcher.mts'].map((file) => path.join(consumer, file)),
    { noEmit: true, strict: true, skipLibCheck: false, types: ['node'], typeRoots: [path.join(consumer, 'node_modules/@types')], target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.NodeNext, moduleResolution: ts.ModuleResolutionKind.NodeNext, esModuleInterop: true });
  const diagnostics = ts.getPreEmitDiagnostics(program);
  assert.equal(diagnostics.length, 0, ts.formatDiagnosticsWithColorAndContext(diagnostics.slice(0, 8), {
    getCurrentDirectory: () => consumer, getCanonicalFileName: (file) => file, getNewLine: () => '\n' }));
  // Unexpected API/source drift must fail before any producer write.
  const before = await readFile(path.join(core, 'dist/vendor/chokidar.d.ts'));
  const manifest = JSON.parse(await readFile(path.join(core, 'package.json'), 'utf8'));
  await put(path.join(core, 'package.json'), JSON.stringify({ ...manifest, dependencies: { ...manifest.dependencies, anymatch: '3.1.2' } }));
  await assert.rejects(prepareSdkTypes(core), /explicit dependency/);
  assert.deepEqual(await readFile(path.join(core, 'dist/vendor/chokidar.d.ts')), before);
  await put(path.join(core, 'package.json'), JSON.stringify(manifest));
  const known = path.join(core, 'dist/config/types.d.ts');
  const knownText = await readFile(known, 'utf8');
  const runtimeImport = knownText.replace('import type { WatchOptions } from', 'import { WatchOptions } from');
  assert.notEqual(runtimeImport, knownText);
  await put(known, runtimeImport);
  await assert.rejects(prepareSdkTypes(core), /Only type-only chokidar imports/);
  assert.deepEqual(await readFile(path.join(core, 'dist/vendor/chokidar.d.ts')), before);
  await put(known, knownText);
  const drift = path.join(temporary, 'drift');
  await put(path.join(drift, 'package.json'), JSON.stringify(manifest));
  const upstreamManifest = JSON.parse(await readFile(chokidarPackage, 'utf8'));
  await put(path.join(drift, 'node_modules/chokidar/package.json'), JSON.stringify({ ...upstreamManifest, version: '3.6.0' }));
  await put(path.join(drift, 'node_modules/anymatch/package.json'), JSON.stringify({ name: 'anymatch', version: '3.1.3' }));
  await assert.rejects(prepareSdkTypes(drift), /changed upstream declarations/);
  await put(path.join(drift, 'node_modules/chokidar/package.json'), JSON.stringify(upstreamManifest));
  await put(path.join(drift, 'node_modules/chokidar/types/index.d.ts'), Buffer.concat([original, Buffer.from('\n')]));
  await put(path.join(drift, 'node_modules/chokidar/LICENSE'), license);
  await assert.rejects(prepareSdkTypes(drift), /Unexpected upstream declaration bytes/);
  await put(path.join(drift, 'node_modules/chokidar/types/index.d.ts'), original);
  await put(path.join(drift, 'node_modules/chokidar/LICENSE'), Buffer.concat([license, Buffer.from('\n')]));
  await assert.rejects(prepareSdkTypes(drift), /Unexpected upstream MIT license bytes/);
  await assert.rejects(access(path.join(drift, 'dist/vendor/chokidar.d.ts')), { code: 'ENOENT' });
  t.diagnostic(JSON.stringify({ temporary, typescript: ts.version, strict: true, skipLibCheck: false,
    diagnostics: 0, negativeAssertions: 13, nodeTypesVersion: JSON.parse(await readFile(path.join(nodeTypes, 'package.json'), 'utf8')).version,
    deterministicPackSha256: hash(await readFile(tarballs[0])),
    runtimeImportedByNormalPrepack: true, prepackEnabled: true, coreRuntimeFabricated: false,
    dependencyInstall: false, originalDeclarationSha256: hash(original), licenseSha256: hash(license) }));
});
