import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { archiveFiles, sha256 } from './release-archive.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const [sealArg, outputArg] = process.argv.slice(2);
if (!sealArg || !outputArg) throw new Error('Specify seal directory and new consumer directory');
const seal = path.resolve(sealArg);
const output = path.resolve(outputArg);
if (fs.existsSync(output)) throw new Error('Preserve existing consumer evidence');
fs.mkdirSync(output, { recursive: true });
fs.writeFileSync(path.join(output, 'package.json'), '{"private":true,"type":"module"}\n');
const receipt = JSON.parse(fs.readFileSync(path.join(seal, 'receipt.json')));
const records = new Map(receipt.records.map(record => [record.name, record]));
const installed = [];
const selected = new Set();
function install(name) {
  if (selected.has(name)) return;
  selected.add(name);
  const record = records.get(name);
  if (!record) throw new Error(`Missing sealed dependency: ${name}`);
  const bytes = fs.readFileSync(path.join(seal, record.file));
  if (sha256(bytes) !== record.sha256) throw new Error('Sealed archive checksum drift');
  const files = archiveFiles(bytes);
  const manifest = JSON.parse(files.get('package/package.json').toString('utf8'));
  const directory = path.join(output, 'node_modules', name);
  for (const [file, content] of files) {
    if (!file.startsWith('package/')) throw new Error('Unexpected npm archive prefix');
    const target = path.resolve(directory, file.slice(8));
    if (!target.startsWith(directory + path.sep)) throw new Error('Unsafe package file');
    fs.mkdirSync(path.dirname(target), { recursive: true }); fs.writeFileSync(target, content);
  }
  installed.push({ name, version: manifest.version, archiveSha256: record.sha256 });
  for (const section of ['dependencies', 'optionalDependencies', 'peerDependencies']) {
    for (const dep of Object.keys(manifest[section] ?? {})) {
      if (!dep.startsWith('@farmfe/')) continue;
      if (section === 'optionalDependencies' && !records.has(dep)) continue;
      install(dep);
    }
  }
}
install('@farmfe/core');
let externalLinks = 0;
function link(name, source) {
  if (name.startsWith('@farmfe/')) return;
  const real = fs.realpathSync(source);
  if (['packages', 'rust-plugins', 'js-plugins', 'examples'].some(group => real.startsWith(path.join(root, group) + path.sep)) || real === root) return;
  const destination = path.join(output, 'node_modules', name);
  if (fs.existsSync(destination)) return;
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.symlinkSync(real, destination, process.platform === 'win32' ? 'junction' : 'dir');
  externalLinks++;
}
for (const modules of [path.join(root, 'packages/core/node_modules'), path.join(root, 'node_modules')]) {
  for (const entry of fs.readdirSync(modules, { withFileTypes: true })) {
    if (entry.name.startsWith('.')) continue;
    const source = path.join(modules, entry.name);
    if (entry.name.startsWith('@')) {
      if (entry.name === '@farmfe') continue;
      for (const name of fs.readdirSync(source)) link(`${entry.name}/${name}`, path.join(source, name));
    } else if (entry.isDirectory() || entry.isSymbolicLink()) link(entry.name, source);
  }
}
const platform = records.get('@farmfe/core-win32-x64-msvc');
if (process.platform !== 'win32' || process.arch !== 'x64' || !platform) throw new Error('This receipt currently validates the Windows x64 candidate only');
const runtimeFile = path.join(output, 'runtime.mjs');
fs.writeFileSync(runtimeFile, `import assert from 'node:assert/strict';import path from 'node:path';import {createRequire} from 'node:module';const esm=await import('@farmfe/core');const cjs=createRequire(import.meta.url)('@farmfe/core');assert.deepEqual(Object.keys(esm).sort(),Object.keys(cjs).sort());for(const module of [esm,cjs]){const identity=await module.compilerNativeIdentity();assert.equal(identity.sha256,${JSON.stringify(platform.nativeSha256)});assert.ok(path.resolve(identity.path).startsWith(${JSON.stringify(path.join(output, 'node_modules', '@farmfe/core-win32-x64-msvc'))}));}console.log(JSON.stringify({esmCjs:true,nativeFromSealedPlatform:true}));\n`);
const env = { ...process.env }; delete env.NAPI_RS_NATIVE_LIBRARY_PATH; delete env.FARM_LIB_CORE_PATH;
const run = spawnSync(process.execPath, [runtimeFile], { cwd: output, env, encoding: 'utf8', timeout: 60000 });
fs.writeFileSync(path.join(output, 'runtime.log'), `${run.stdout}\n${run.stderr}`);
if (run.status !== 0) throw new Error(`Sealed runtime failed: ${run.stderr}`);
const contract = `import {defineConfig,type UserConfig,type JsPlugin,type Compiler} from '@farmfe/core';\nconst config:UserConfig=defineConfig({compilation:{input:{index:'./entry.ts'},output:{path:'./dist'}}});\nconst plugin:JsPlugin={name:'external-contract'};declare const compiler:Compiler;void compiler;void config;void plugin;\n// @ts-expect-error input entries require string paths\nconst badInput:UserConfig={compilation:{input:{index:42}}};\n// @ts-expect-error port must be numeric\nconst badPort:UserConfig={server:{port:'bad'}};\n// @ts-expect-error plugin names must be strings\nconst badPlugin:JsPlugin={name:17};void badInput;void badPort;void badPlugin;\n`;
for (const extension of ['mts', 'cts']) fs.writeFileSync(path.join(output, `consumer.${extension}`), contract);
const require = createRequire(import.meta.url);
const ts = require('typescript');
const program = ts.createProgram(['mts', 'cts'].map(extension => path.join(output, `consumer.${extension}`)), { noEmit: true, strict: true, skipLibCheck: false, target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.NodeNext, moduleResolution: ts.ModuleResolutionKind.NodeNext, types: ['node'], typeRoots: [path.join(output, 'node_modules/@types')], esModuleInterop: true });
const diagnostics = ts.getPreEmitDiagnostics(program);
if (diagnostics.length) throw new Error(ts.formatDiagnosticsWithColorAndContext(diagnostics, { getCurrentDirectory: () => output, getCanonicalFileName: file => file, getNewLine: () => '\n' }));
fs.writeFileSync(path.join(output, 'result.json'), JSON.stringify({ status: 'passed', scope: 'Npm Core ESM/CJS/native and strict NodeNext declarations; not Cargo Compiler/Toolkit or public registry installation', sourceCommit: receipt.sourceCommit, packages: installed, externalRegistryCacheLinks: externalLinks, workspacePackageLinks: 0, nativeOverrides: false, strict: true, skipLibCheck: false, negativeTypeAssertions: 6, diagnostics: 0 }, null, 2));
console.log(JSON.stringify({ status: 'passed', sealedFarmPackages: installed.length, workspacePackageLinks: 0, strictDiagnostics: 0 }));
