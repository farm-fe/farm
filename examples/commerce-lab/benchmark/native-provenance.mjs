import fs from 'node:fs/promises';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import { fileURLToPath, pathToFileURL } from 'node:url';

const require = createRequire(import.meta.url);
const root = fileURLToPath(new URL('../../../', import.meta.url));
export async function fingerprintNativeFile(filename) {
  const realPath = await fs.realpath(filename);
  const bytes = await fs.readFile(realPath);
  return { path: realPath, bytes: bytes.length, sha256: createHash('sha256').update(bytes).digest('hex') };
}

export async function captureNativeProvenance() {
  const artifacts = {};
  if (process.env.NAPI_RS_FORCE_WASI) throw new Error('Native provenance does not yet support forced WASI bindings');
  const coreRequire = createRequire(path.join(root, 'packages/core/binding/binding.cjs'));
  const corePath = process.env.NAPI_RS_NATIVE_LIBRARY_PATH
    ? coreRequire.resolve(process.env.NAPI_RS_NATIVE_LIBRARY_PATH)
    : require(path.join(root, 'packages/core/binding/resolve-binding.cjs'));
  artifacts['@farmfe/core'] = await fingerprintNativeFile(corePath);
  for (const name of ['replace-dirname', 'react', 'worker', 'auto-import', 'svgr', 'sass', 'yaml', 'dsv', 'mdx', 'tailwindcss', 'compress', 'dts', 'vue', 'vue-jsx', 'wasm']) {
    const pkg = `@farmfe/plugin-${name}`;
    const packagePath = (name === 'replace-dirname' ? coreRequire : require).resolve(`${pkg}/package.json`);
    const loader = path.join(path.dirname(packagePath), 'scripts/index.js');
    const module = await import(pathToFileURL(loader).href);
    artifacts[pkg] = { ...await fingerprintNativeFile(module.default), loader: await fs.realpath(loader) };
  }
  const cargoLock = await fs.readFile(path.join(root, 'Cargo.lock'), 'utf8');
  const virtualLoader = path.join(root, 'rust-plugins/virtual/scripts/index.js');
  const virtualModule = await import(pathToFileURL(virtualLoader).href);
  artifacts['@farmfe/plugin-virtual'] = { ...await fingerprintNativeFile(virtualModule.default), loader: await fs.realpath(virtualLoader) };
  const urlLoader = path.join(root, 'rust-plugins/url/scripts/index.js');
  const urlModule = await import(pathToFileURL(urlLoader).href);
  artifacts['@farmfe/plugin-url'] = { ...await fingerprintNativeFile(urlModule.default), loader: await fs.realpath(urlLoader) };
  const imageLoader = path.join(root, 'rust-plugins/image/scripts/index.js');
  const imageModule = await import(pathToFileURL(imageLoader).href);
  artifacts['@farmfe/plugin-image'] = { ...await fingerprintNativeFile(imageModule.default), loader: await fs.realpath(imageLoader) };
  const iconsLoader = path.join(root, 'rust-plugins/icons/scripts/index.js');
  const iconsModule = await import(pathToFileURL(iconsLoader).href);
  artifacts['@farmfe/plugin-icons'] = { ...await fingerprintNativeFile(iconsModule.default), loader: await fs.realpath(iconsLoader) };
  const stripLoader = path.join(root, 'rust-plugins/strip/scripts/index.js');
  const stripModule = await import(pathToFileURL(stripLoader).href);
  artifacts['@farmfe/plugin-strip'] = { ...await fingerprintNativeFile(stripModule.default), loader: await fs.realpath(stripLoader) };
  const modularLoader = path.join(root, 'rust-plugins/modular-import/scripts/index.js');
  const modularModule = await import(pathToFileURL(modularLoader).href);
  artifacts['@farmfe/plugin-modular-import'] = { ...await fingerprintNativeFile(modularModule.default), loader: await fs.realpath(modularLoader) };
  const componentsLoader = path.join(root, 'rust-plugins/react-components/scripts/index.js');
  const componentsModule = await import(pathToFileURL(componentsLoader).href);
  artifacts['@farmfe/plugin-react-components'] = { ...await fingerprintNativeFile(componentsModule.default), loader: await fs.realpath(componentsLoader) };
  const rustDependencies = cargoLock.split('[[package]]').flatMap(block => {
    const name = /^name = "([^"]+)"/m.exec(block)?.[1];
    if (!name || !/^(swc_ecma_(parser|ast)|fervid.*|grass|lightningcss|serde_yaml|csv|mdx.*)$/.test(name)) return [];
    return [{ name, version: /^version = "([^"]+)"/m.exec(block)?.[1], source: /^source = "([^"]+)"/m.exec(block)?.[1] ?? 'workspace' }];
  });
  const wasmPackage = require.resolve('json_typegen_wasm/package.json');
  const wasmInput = await fingerprintNativeFile(path.join(path.dirname(wasmPackage), 'json_typegen_wasm_bg.wasm'));
  return { artifacts, wasmInput, cargoLockSha256: createHash('sha256').update(cargoLock).digest('hex'), rustDependencies,
    limitation: 'Hashes identify selected on-disk native artifacts. Cargo.lock describes source resolution; it does not prove which dependencies were embedded in a previously built artifact.' };
}
