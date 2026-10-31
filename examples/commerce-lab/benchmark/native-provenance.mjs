import fs from 'node:fs/promises';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import { fileURLToPath, pathToFileURL } from 'node:url';

const require = createRequire(import.meta.url);
const root = fileURLToPath(new URL('../../../', import.meta.url));
const regularPlugins = ['replace-dirname', 'react', 'worker', 'auto-import', 'svgr', 'sass', 'yaml', 'dsv', 'mdx', 'tailwindcss', 'compress', 'dts', 'vue', 'vue-jsx', 'wasm'];
const localPlugins = ['virtual', 'url', 'image', 'icons', 'strip', 'modular-import', 'react-components'];

export function nativeProvenanceScope(options = {}) {
  const groups = options.groups ?? ['js', 'rust', 'mixed'];
  if (!Array.isArray(groups) || !groups.length || groups.some(group => !['js', 'rust', 'mixed'].includes(group)) || new Set(groups).size !== groups.length) {
    throw new Error('Invalid native provenance groups');
  }
  const jsOnly = groups.length === 1 && groups[0] === 'js' && options.configOverride === undefined && options.npmProject === undefined;
  return {
    kind: jsOnly ? 'default-js-config' : 'complete-lab',
    groups: [...groups],
    plugins: jsOnly ? ['replace-dirname', 'react', 'worker'] : [...regularPlugins, ...localPlugins],
  };
}
export async function fingerprintNativeFile(filename) {
  const realPath = await fs.realpath(filename);
  const bytes = await fs.readFile(realPath);
  return { path: realPath, bytes: bytes.length, sha256: createHash('sha256').update(bytes).digest('hex') };
}

export async function captureNativeProvenance(options = {}) {
  const scope = nativeProvenanceScope(options);
  const artifacts = {};
  if (process.env.NAPI_RS_FORCE_WASI) throw new Error('Native provenance does not yet support forced WASI bindings');
  const coreRequire = createRequire(path.join(root, 'packages/core/binding/binding.cjs'));
  const corePath = process.env.NAPI_RS_NATIVE_LIBRARY_PATH
    ? coreRequire.resolve(process.env.NAPI_RS_NATIVE_LIBRARY_PATH)
    : require(path.join(root, 'packages/core/binding/resolve-binding.cjs'));
  artifacts['@farmfe/core'] = await fingerprintNativeFile(corePath);
  for (const name of regularPlugins.filter(name => scope.plugins.includes(name))) {
    const pkg = `@farmfe/plugin-${name}`;
    const packagePath = (name === 'replace-dirname' ? coreRequire : require).resolve(`${pkg}/package.json`);
    const loader = path.join(path.dirname(packagePath), 'scripts/index.js');
    const module = await import(pathToFileURL(loader).href);
    artifacts[pkg] = { ...await fingerprintNativeFile(module.default), loader: await fs.realpath(loader) };
  }
  const cargoLock = await fs.readFile(path.join(root, 'Cargo.lock'), 'utf8');
  for (const name of localPlugins.filter(name => scope.plugins.includes(name))) {
    const loader = path.join(root, 'rust-plugins', name, 'scripts/index.js');
    const module = await import(pathToFileURL(loader).href);
    artifacts[`@farmfe/plugin-${name}`] = { ...await fingerprintNativeFile(module.default), loader: await fs.realpath(loader) };
  }
  const rustDependencies = cargoLock.split('[[package]]').flatMap(block => {
    const name = /^name = "([^"]+)"/m.exec(block)?.[1];
    if (!name || !/^(swc_ecma_(parser|ast)|fervid.*|grass|lightningcss|serde_yaml|csv|mdx.*)$/.test(name)) return [];
    return [{ name, version: /^version = "([^"]+)"/m.exec(block)?.[1], source: /^source = "([^"]+)"/m.exec(block)?.[1] ?? 'workspace' }];
  });
  const wasmPackage = require.resolve('json_typegen_wasm/package.json');
  const wasmInput = await fingerprintNativeFile(path.join(path.dirname(wasmPackage), 'json_typegen_wasm_bg.wasm'));
  return { scope, artifacts, wasmInput, cargoLockSha256: createHash('sha256').update(cargoLock).digest('hex'), rustDependencies,
    limitation: 'Hashes identify selected on-disk native artifacts. Cargo.lock describes source resolution; it does not prove which dependencies were embedded in a previously built artifact.' };
}
