// cspell:words glibc
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';

// No CLI or environment override: tests inject the Python smoke function.
assert.equal(process.version, 'v22.20.0', 'Registry host requires the pinned production Node version');
assert.equal(process.platform, 'linux');
assert.equal(process.arch, 'x64');
assert.match(process.report.getReport().header.glibcVersionRuntime ?? '', /^\d+\.\d+/);
const [binary, runtime] = process.argv.slice(2);
assert.ok(path.isAbsolute(binary) && path.isAbsolute(runtime));
const require = createRequire(import.meta.url);
const native = require(binary);
const requiredExports = ['Compiler', 'DocsSnapshot', 'OutputDirectoryLock', 'Resolver'];
for (const name of requiredExports) assert.equal(typeof native[name], 'function', `Missing native ${name}`);
for (const name of ['compile', 'compileSync', 'resources', 'inputObservations', 'flushCache', 'traceModuleGraph']) {
  assert.equal(typeof native.Compiler.prototype[name], 'function', `Missing compiler ${name}`);
}
const root = process.cwd();
const input = path.join(root, 'registry-host-smoke.ts');
fs.writeFileSync(input, 'const value: number = 42; export { value };\n', { flag: 'wx' });
const config = {
  root, input: { index: './registry-host-smoke.ts' },
  output: { path: path.join(root, 'out'), format: 'cjs', targetEnv: 'library', entryFilename: 'index.cjs' },
  runtime: { path: runtime, plugins: [] }, mode: 'production', persistentCache: false,
  progress: false, presetEnv: false, minify: false, sourcemap: false,
  lazyCompilation: false, concatenateModules: true, treeShaking: false
};
const resolver = new native.Resolver(config);
assert.equal(resolver.resolve('./registry-host-smoke.ts', root), input);
const compiler = new native.Compiler({ config, jsPlugins: [], rustPlugins: [] });
compiler.compileSync();
const resources = compiler.resources();
assert.deepEqual(Object.keys(resources), ['index.cjs']);
const bytes = Buffer.from(resources['index.cjs']);
assert.ok(bytes.length > 0 && bytes.length <= 2 * 1024 ** 2);
assert.match(bytes.toString(), /42/);
assert.doesNotMatch(bytes.toString(), /: number/);
const output = path.join(root, 'registry-host-result.cjs');
fs.writeFileSync(output, bytes, { flag: 'wx' });
assert.equal(require(output).value, 42);
console.log('FARM_REGISTRY_HOST_SMOKE=' + JSON.stringify({
  node: process.version, napi: process.versions.napi, platform: process.platform,
  arch: process.arch, libc: 'glibc', requiredExports,
  resolver: true, compile: true, tsTransformed: true, outputExecuted: true,
  outputBytes: bytes.length, outputSha256: crypto.createHash('sha256').update(bytes).digest('hex')
}));
