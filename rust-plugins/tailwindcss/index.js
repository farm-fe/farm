import { existsSync, readFileSync } from 'fs';
import moduleBuiltin, { createRequire } from 'module';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const { platform, arch } = process;
const currentDir = dirname(fileURLToPath(import.meta.url));

let binPath = null;

const require = createRequire(import.meta.url);
const Module = moduleBuiltin.Module;
const TAILWIND_CONFIG_FILE = 'tailwindcss.config.js';

function isMusl() {
  if (!process.report || typeof process.report.getReport !== 'function') {
    try {
      return readFileSync('/usr/bin/ldd', 'utf8').includes('musl');
    } catch (e) {
      return true;
    }
  } else {
    const { glibcVersionRuntime } = process.report.getReport().header;
    return !glibcVersionRuntime;
  }
}

switch (platform) {
  case 'win32':
    switch (arch) {
      case 'x64':
        if (existsSync(join(currentDir, './npm/win32-x64-msvc/index.farm'))) {
          binPath = join(currentDir, './npm/win32-x64-msvc/index.farm');
        } else {
          binPath = require.resolve(
            '@farmfe/plugin-tailwindcss-win32-x64-msvc'
          );
        }
        break;
      case 'ia32':
        if (existsSync(join(currentDir, './npm/win32-ia32-msvc/index.farm'))) {
          binPath = join(currentDir, './npm/win32-ia32-msvc/index.farm');
        } else {
          binPath = require.resolve(
            '@farmfe/plugin-tailwindcss-win32-ia32-msvc'
          );
        }
        break;
      case 'arm64':
        if (
          existsSync(join(currentDir, './npm/win32-arm64-msvc/index.farm'))
        ) {
          binPath = join(currentDir, './npm/win32-arm64-msvc/index.farm');
        } else {
          binPath = require.resolve(
            '@farmfe/plugin-tailwindcss-win32-arm64-msvc'
          );
        }
        break;
      default:
        throw new Error(`Unsupported architecture on Windows: ${arch}`);
    }
    break;
  case 'darwin':
    switch (arch) {
      case 'x64':
        if (existsSync(join(currentDir, './npm/darwin-x64/index.farm'))) {
          binPath = join(currentDir, './npm/darwin-x64/index.farm');
        } else {
          binPath = require.resolve('@farmfe/plugin-tailwindcss-darwin-x64');
        }
        break;
      case 'arm64':
        if (existsSync(join(currentDir, './npm/darwin-arm64/index.farm'))) {
          binPath = join(currentDir, './npm/darwin-arm64/index.farm');
        } else {
          binPath = require.resolve('@farmfe/plugin-tailwindcss-darwin-arm64');
        }
        break;
      default:
        throw new Error(`Unsupported architecture on macOS: ${arch}`);
    }
    break;
  case 'linux':
    switch (arch) {
      case 'x64':
        if (isMusl()) {
          if (
            existsSync(join(currentDir, './npm/linux-x64-musl/index.farm'))
          ) {
            binPath = join(currentDir, './npm/linux-x64-musl/index.farm');
          } else {
            binPath = require.resolve(
              '@farmfe/plugin-tailwindcss-linux-x64-musl'
            );
          }
        } else {
          if (
            existsSync(join(currentDir, './npm/linux-x64-gnu/index.farm'))
          ) {
            binPath = join(currentDir, './npm/linux-x64-gnu/index.farm');
          } else {
            binPath = require.resolve(
              '@farmfe/plugin-tailwindcss-linux-x64-gnu'
            );
          }
        }
        break;
      case 'arm64':
        if (isMusl()) {
          if (
            existsSync(join(currentDir, './npm/linux-arm64-musl/index.farm'))
          ) {
            binPath = join(currentDir, './npm/linux-arm64-musl/index.farm');
          } else {
            binPath = require.resolve(
              '@farmfe/plugin-tailwindcss-linux-arm64-musl'
            );
          }
        } else {
          if (
            existsSync(join(currentDir, './npm/linux-arm64-gnu/index.farm'))
          ) {
            binPath = join(currentDir, './npm/linux-arm64-gnu/index.farm');
          } else {
            binPath = require.resolve(
              '@farmfe/plugin-tailwindcss-linux-arm64-gnu'
            );
          }
        }
        break;
      default:
        throw new Error(`Unsupported architecture on Linux: ${arch}`);
    }
    break;
  default:
    throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`);
}

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
