import { existsSync, readFileSync } from 'fs';
import moduleBuiltin, { createRequire } from 'module';
import { dirname, join } from 'path';

import binPath from './index.js';

const require = createRequire(import.meta.url);
const Module = moduleBuiltin.Module;
const TAILWIND_CONFIG_FILE = 'tailwindcss.config.js';

function isJsonSerializable(value, seen = new WeakSet()) {
  if (value == null) {
    return true;
  }

  const valueType = typeof value;
  if (valueType === 'string' || valueType === 'number' || valueType === 'boolean') {
    return true;
  }

  if (valueType !== 'object') {
    return false;
  }

  if (seen.has(value)) {
    return false;
  }
  seen.add(value);

  if (Array.isArray(value)) {
    return value.every((item) => isJsonSerializable(item, seen));
  }

  return Object.values(value).every((item) => isJsonSerializable(item, seen));
}

function loadTailwindConfig() {
  const configPath = join(process.cwd(), TAILWIND_CONFIG_FILE);
  if (!existsSync(configPath)) {
    return undefined;
  }

  const config = loadConfigModule(configPath);
  const resolvedConfig =
    config && typeof config === 'object' && 'default' in config
      ? config.default
      : config;

  if (!isJsonSerializable(resolvedConfig)) {
    throw new Error(
      `@farmfe/plugin-tailwindcss only supports JSON-serializable ${TAILWIND_CONFIG_FILE} exports`
    );
  }

  return JSON.parse(JSON.stringify(resolvedConfig));
}

function loadConfigModule(configPath) {
  try {
    delete require.cache[configPath];
    return require(configPath);
  } catch (error) {
    if (!shouldLoadConfigFromSource(error)) {
      throw error;
    }

    return loadConfigFromSource(configPath);
  }
}

function shouldLoadConfigFromSource(error) {
  if (error && typeof error === 'object' && error.code === 'ERR_REQUIRE_ESM') {
    return true;
  }

  return (
    error instanceof SyntaxError &&
    /Unexpected token (?:'export'|export)/.test(error.message)
  );
}

function loadConfigFromSource(configPath) {
  const source = readFileSync(configPath, 'utf8');
  const compiledSource = source.replace(
    /^(\s*)export\s+default\b/m,
    '$1module.exports ='
  );
  const mod = new Module(configPath);
  mod.filename = configPath;
  mod.paths = Module._nodeModulePaths(dirname(configPath));
  mod._compile(compiledSource, configPath);
  return mod.exports;
}

export default function tailwindcss(options = {}) {
  const loadedConfig =
    options.config === undefined ? loadTailwindConfig() : undefined;

  return [
    binPath,
    loadedConfig === undefined
      ? options
      : {
          ...options,
          config: loadedConfig
        }
  ];
}
