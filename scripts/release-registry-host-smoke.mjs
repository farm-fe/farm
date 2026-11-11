// cspell:words glibc
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

// No CLI or environment override: tests inject the Python smoke function.
assert.equal(process.version, 'v22.20.0', 'Registry host requires the pinned production Node version');
assert.equal(process.platform, 'linux');
assert.equal(process.arch, 'x64');
assert.match(process.report.getReport().header.glibcVersionRuntime ?? '', /^\d+\.\d+/);
const [binary, runtime, plugin, packageName] = process.argv.slice(2);
assert.ok(path.isAbsolute(binary) && path.isAbsolute(runtime));
const replaceDirname = packageName === '@farmfe/plugin-replace-dirname-linux-x64-gnu';
const worker = packageName === '@farmfe/plugin-worker-linux-x64-gnu';
assert.ok(plugin ? path.isAbsolute(plugin) && (replaceDirname || worker) : packageName === undefined);
const require = createRequire(import.meta.url);
const native = require(binary);
const requiredExports = ['Compiler', 'DocsSnapshot', 'OutputDirectoryLock', 'Resolver'];
for (const name of requiredExports) assert.equal(typeof native[name], 'function', `Missing native ${name}`);
for (const name of ['compile', 'compileSync', 'resources', 'inputObservations', 'flushCache', 'traceModuleGraph']) {
  assert.equal(typeof native.Compiler.prototype[name], 'function', `Missing compiler ${name}`);
}
const root = process.cwd();
// A different source directory ensures untransformed __dirname cannot pass.
const sourceDirectory = path.join(root, 'fixture-source');
fs.mkdirSync(sourceDirectory);
const input = path.join(sourceDirectory, 'registry-host-smoke.ts');
let source = 'const value: number = 42; export { value };\n';
if (replaceDirname) source += 'export const location = { directory: __dirname, file: __filename, url: import.meta.url };\n';
if (worker) {
  source += "import WorkerFactory from './registry-fixture-worker.ts?worker&inline';\nexport function makeWorker(options) { return new WorkerFactory(options); }\n";
  fs.writeFileSync(path.join(sourceDirectory, 'registry-fixture-worker.ts'),
    'const marker: string = "FARM_REGISTRY_WORKER_FIXTURE"; self.postMessage(marker);\n', { flag: 'wx' });
}
fs.writeFileSync(input, source, { flag: 'wx' });
const config = {
  root, input: { index: './fixture-source/registry-host-smoke.ts' },
  output: { path: path.join(root, 'out'), format: 'cjs', targetEnv: 'library', entryFilename: '[entryName].cjs' },
  runtime: { path: runtime, plugins: [] }, mode: 'production', persistentCache: false,
  progress: false, presetEnv: false, minify: false, sourcemap: false,
  lazyCompilation: false, concatenateModules: true, treeShaking: false
};
const resolver = new native.Resolver(config);
assert.equal(resolver.resolve('./fixture-source/registry-host-smoke.ts', root), input);
// This is the actual Farm dynamic-plugin loader and ABI boundary, not dlopen
// alone or an invented exported symbol check. A mismatch fails this process.
const compiler = new native.Compiler({ config, jsPlugins: [], rustPlugins: plugin ? [[plugin, '{}']] : [] });
compiler.compileSync();
const resources = compiler.resources();
assert.ok(Object.hasOwn(resources, 'index.cjs'));
if (!worker) assert.deepEqual(Object.keys(resources), ['index.cjs']);
assert.ok(Object.keys(resources).length <= 10);
const bytes = Buffer.from(resources['index.cjs']);
assert.ok(bytes.length > 0 && bytes.length <= 2 * 1024 ** 2);
assert.match(bytes.toString(), /42/);
assert.doesNotMatch(bytes.toString(), /: number/);
const output = path.join(root, 'registry-host-result.cjs');
fs.writeFileSync(output, bytes, { flag: 'wx' });
const result = require(output);
assert.equal(result.value, 42);
if (replaceDirname) {
  assert.deepEqual(result.location, { directory: sourceDirectory, file: input, url: pathToFileURL(input).href });
}
if (worker) {
  // No actual browser/worker or network access is needed. The compiled wrapper
  // must construct a Worker using a fully bundled, TypeScript-free inline body.
  const previous = globalThis.Worker;
  globalThis.Worker = class { constructor(url, options) { this.url = url; this.options = options; } };
  try {
    const created = result.makeWorker({ name: 'registry-host-fixture' });
    assert.deepEqual(created.options, { name: 'registry-host-fixture' });
    assert.match(created.url, /^data:text\/javascript;base64,[A-Za-z0-9+/]+=*$/);
    const compiledWorker = Buffer.from(created.url.split(',')[1], 'base64');
    assert.ok(compiledWorker.length > 0 && compiledWorker.length <= 2 * 1024 ** 2);
    assert.match(compiledWorker.toString(), /FARM_REGISTRY_WORKER_FIXTURE/);
    assert.doesNotMatch(compiledWorker.toString(), /: string|__FARM_INLINE_WORKER:/);
    assert.ok(Object.values(resources).some(resource => Buffer.from(resource).equals(compiledWorker)),
      'Inline worker must contain a real compiler resource');
  } finally {
    if (previous === undefined) delete globalThis.Worker;
    else globalThis.Worker = previous;
  }
}
console.log('FARM_REGISTRY_HOST_SMOKE=' + JSON.stringify({
  node: process.version, napi: process.versions.napi, platform: process.platform,
  arch: process.arch, libc: 'glibc', requiredExports,
  resolver: true, compile: true, tsTransformed: true, outputExecuted: true,
  ...(plugin ? { plugin: packageName, rustPluginLoaded: true, pluginFixture: true } : {}),
  outputBytes: bytes.length, outputSha256: crypto.createHash('sha256').update(bytes).digest('hex')
}));
