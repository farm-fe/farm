import { createHash } from 'node:crypto';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import fse from 'fs-extra';

import { bindingPath } from '../../binding/index.js';
import { OutputConfig } from '../types/binding.js';

import { JsPlugin } from '../index.js';
import {
  getSortedPlugins,
  handleVitePlugins,
  resolveAsyncPlugins,
  resolveConfigHook,
  resolveConfigResolvedHook,
  resolveFarmPlugins
} from '../plugin/index.js';
import {
  Logger,
  clearScreen,
  colors,
  isArray,
  isEmptyObject,
  isObject,
  isWindows,
  normalizePath
} from '../utils/index.js';
import { traceDependencies } from '../utils/trace-dependencies.js';
import { __FARM_GLOBAL__ } from './_global.js';
import {
  CompilationMode,
  getExistsEnvFiles,
  loadEnv,
  setProcessEnv
} from './env.js';
import {
  getValidPublicPath,
  normalizeOutput
} from './normalize-config/normalize-output.js';
import { normalizePersistentCache } from './normalize-config/normalize-persistent-cache.js';
import { parseUserConfig } from './schema.js';

import { externalAdapter } from '../plugin/js/external-adapter.js';
import { convertErrorMessage } from '../utils/error.js';
import { resolveHostname } from '../utils/http.js';
import merge from '../utils/merge.js';
import {
  CUSTOM_KEYS,
  DEFAULT_CONFIG_NAMES,
  FARM_DEFAULT_NAMESPACE
} from './constants.js';
import { mergeConfig, mergeFarmCliConfig } from './mergeConfig.js';
import { normalizeAsset } from './normalize-config/normalize-asset.js';
import { normalizeCss } from './normalize-config/normalize-css.js';
import { normalizeExternal } from './normalize-config/normalize-external.js';
import normalizePartialBundling from './normalize-config/normalize-partial-bundling.js';
import { normalizeResolve } from './normalize-config/normalize-resolve.js';
import type {
  ConfigEnv,
  FarmCliOptions,
  NormalizedServerConfig,
  ResolvedCompilation,
  ResolvedUserConfig,
  UserConfig,
  UserConfigExport,
  UserConfigFnObject,
  UserHmrConfig,
  UserServerConfig
} from './types.js';

export * from './types.js';
export * from './constants.js';

export function defineFarmConfig(config: UserConfig): UserConfig;
export function defineFarmConfig(
  config: Promise<UserConfig>
): Promise<UserConfig>;
export function defineFarmConfig(
  config: UserConfigFnObject
): UserConfigFnObject;
export function defineFarmConfig(config: UserConfigExport): UserConfigExport;
export function defineFarmConfig(config: UserConfigExport): UserConfigExport {
  return config;
}

const COMMANDS = {
  START: 'start',
  BUILD: 'build',
  WATCH: 'watch',
  PREVIEW: 'preview',
  CLEAN: 'clean'
} as const;

/**
 * Resolve and load user config from the specified path
 * @param configPath
 */
export async function resolveConfig(
  inlineOptions: FarmCliOptions & UserConfig,
  command: 'start' | 'build' | 'watch' | 'preview',
  defaultMode: CompilationMode = 'development',
  defaultNodeEnv: CompilationMode = 'development',
  isPreview = false
): Promise<ResolvedUserConfig> {
  // TODO mode 这块还是不对 要区分 mode 和 build 还是 dev 环境
  // TODO 在使用 vite 插件的时候 不要在开发环境使用 生产环境的mode vue 插件会导致 hmr 失效 记在文档里
  const compileMode = defaultMode;

  const mode = inlineOptions.mode || defaultMode;
  const isNodeEnvSet = !!process.env.NODE_ENV;
  inlineOptions.mode = mode;

  if (!isNodeEnvSet) {
    setProcessEnv(defaultNodeEnv);
  }

  const configEnv: ConfigEnv = {
    mode,
    command,
    isPreview
  };

  // configPath may be file or directory
  const { configFile, configPath: initialConfigPath } = inlineOptions;

  const loadedUserConfig = await loadConfigFile(
    configFile,
    inlineOptions,
    configEnv,
    defaultNodeEnv
  );

  let rawConfig: UserConfig = mergeFarmCliConfig(
    inlineOptions,
    {},
    compileMode
  );

  const inlineConfig = rawConfig;

  let configFilePath = initialConfigPath;

  if (loadedUserConfig) {
    configFilePath = loadedUserConfig.configFilePath;
    rawConfig = mergeConfig(rawConfig, loadedUserConfig.config);
  }

  const { jsPlugins, rustPlugins, vitePluginAdapters } = await resolvePlugins(
    rawConfig,
    compileMode
  );

  const sortFarmJsPlugins = getSortedPlugins([
    ...jsPlugins,
    ...vitePluginAdapters,
    externalAdapter()
  ]);

  const config = await resolveConfigHook(rawConfig, sortFarmJsPlugins);
  // define logger when resolvedConfigHook
  const logger = new Logger({
    customLogger: loadedUserConfig.config?.customLogger,
    allowClearScreen: loadedUserConfig.config?.clearScreen
  });

  const resolvedUserConfig = await resolveUserConfig(
    config,
    configFilePath,
    compileMode
  );

  resolvedUserConfig.logger = logger;

  // normalize server config first cause it may be used in normalizeUserCompilationFnConfig
  resolvedUserConfig.server = normalizeDevServerConfig(
    resolvedUserConfig.server,
    compileMode
  );

  resolvedUserConfig.compilation = await normalizeUserCompilationConfig(
    resolvedUserConfig,
    mode as CompilationMode
  );

  Object.assign(resolvedUserConfig, {
    root: resolvedUserConfig.compilation.root,
    jsPlugins: sortFarmJsPlugins,
    rustPlugins: rustPlugins,
    inlineConfig
  });

  await resolveConfigResolvedHook(resolvedUserConfig, sortFarmJsPlugins); // Fix: Await the Promise<void> and pass the resolved value to the function.

  await handleLazyCompilation(
    resolvedUserConfig,
    command as keyof typeof COMMANDS
  );

  return resolvedUserConfig;
}

async function handleLazyCompilation(
  config: ResolvedUserConfig,
  command: keyof typeof COMMANDS
) {
  const commandHandlers = {
    [COMMANDS.START]: async (cfg: ResolvedUserConfig) => {
      if (
        cfg.compilation.lazyCompilation &&
        typeof cfg.server?.host === 'string'
      ) {
        await setLazyCompilationDefine(cfg);
      }
    },
    [COMMANDS.WATCH]: async (cfg: ResolvedUserConfig) => {
      if (cfg.compilation?.lazyCompilation) {
        await setLazyCompilationDefine(cfg);
      }
    }
  };

  const handler = commandHandlers[command as keyof typeof commandHandlers];
  if (handler) {
    await handler(config);
  }
}

/**
 * Normalize user config and transform it to rust compiler compatible config
 *
 *
 * ResolvedUserConfig is a parameter passed to rust Compiler,
 * and ResolvedUserConfig is generated from UserConfig.
 * When UserConfig is different from ResolvedUserConfig,
 * a legal value should be given to the ResolvedUserConfig field here,
 * and converted from UserConfig in the subsequent process.
 *
 * @param config
 * @returns resolved config that parsed to rust compiler
 */
export async function normalizeUserCompilationConfig(
  resolvedUserConfig: ResolvedUserConfig,
  mode: CompilationMode = 'development'
): Promise<ResolvedCompilation> {
  const { compilation, root } = resolvedUserConfig;

  // resolve root path

  const resolvedRootPath = normalizePath(root);

  resolvedUserConfig.root = resolvedRootPath;

  // if normalize default config, skip check input option
  const inputIndexConfig = await checkCompilationInputValue(
    resolvedUserConfig,
    resolvedUserConfig.logger
  );

  const resolvedCompilation: ResolvedCompilation = merge(
    {},
    DEFAULT_COMPILATION_OPTIONS,
    {
      input: inputIndexConfig,
      root: resolvedRootPath
    },
    compilation
  );

  const isProduction = mode === 'production';
  const isDevelopment = mode === 'development';
  resolvedCompilation.mode = resolvedCompilation.mode ?? mode;

  resolvedCompilation.coreLibPath = bindingPath;

  normalizeOutput(resolvedCompilation, isProduction, resolvedUserConfig.logger);
  normalizeExternal(resolvedUserConfig, resolvedCompilation);

  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore do not check type for this internal option
  if (!resolvedCompilation.assets?.publicDir) {
    resolvedCompilation.assets ??= {};

    const userPublicDir = resolvedUserConfig.publicDir
      ? resolvedUserConfig.publicDir
      : path.join(resolvedCompilation.root, 'public');

    if (path.isAbsolute(userPublicDir)) {
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore do not check type for this internal option
      resolvedCompilation.assets.publicDir = userPublicDir;
    } else {
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore do not check type for this internal option
      resolvedCompilation.assets.publicDir = path.join(
        resolvedCompilation.root,
        userPublicDir
      );
    }
  }

  resolvedCompilation.define = Object.assign(
    {
      // skip self define
      ['FARM' + '_PROCESS_ENV']: resolvedUserConfig.env,
      FARM_RUNTIME_TARGET_ENV: JSON.stringify(
        resolvedCompilation.output?.targetEnv
      )
    },
    resolvedCompilation?.define,
    // for node target, we should not define process.env.NODE_ENV
    resolvedCompilation.output?.targetEnv === 'node'
      ? {}
      : Object.keys(resolvedUserConfig.env || {}).reduce((env: any, key) => {
          env[`$__farm_regex:(global(This)?\\.)?process\\.env\\.${key}`] =
            JSON.stringify(resolvedUserConfig.env[key]);
          return env;
        }, {})
  );

  const require = createRequire(import.meta.url);
  const hmrClientPluginPath = require.resolve('@farmfe/runtime-plugin-hmr');
  const ImportMetaPluginPath = require.resolve(
    '@farmfe/runtime-plugin-import-meta'
  );

  if (!resolvedCompilation.runtime) {
    resolvedCompilation.runtime = {
      path: require.resolve('@farmfe/runtime'),
      plugins: []
    };
  }

  if (!resolvedCompilation.runtime.path) {
    resolvedCompilation.runtime.path = require.resolve('@farmfe/runtime');
  }

  if (!resolvedCompilation.runtime.swcHelpersPath) {
    resolvedCompilation.runtime.swcHelpersPath = path.dirname(
      require.resolve('@swc/helpers/package.json')
    );
  }

  if (!resolvedCompilation.runtime.plugins) {
    resolvedCompilation.runtime.plugins = [];
  } else {
    const resolvePluginPath = (plugin: any) => {
      if (path.isAbsolute(plugin)) return plugin;
      return plugin.startsWith('.')
        ? path.resolve(resolvedRootPath, plugin)
        : require.resolve(plugin);
    };
    // make sure all plugin paths are absolute
    resolvedCompilation.runtime.plugins =
      resolvedCompilation.runtime.plugins.map(resolvePluginPath);
  }
  // set namespace to package.json name field's hash
  if (!resolvedCompilation.runtime.namespace) {
    // read package.json name field
    const packageJsonPath = path.resolve(resolvedRootPath, 'package.json');
    const packageJsonExists = fse.existsSync(packageJsonPath);
    const namespaceName = packageJsonExists
      ? JSON.parse(fse.readFileSync(packageJsonPath, 'utf-8')).name ||
        FARM_DEFAULT_NAMESPACE
      : FARM_DEFAULT_NAMESPACE;

    resolvedCompilation.runtime.namespace = createHash('md5')
      .update(namespaceName)
      .digest('hex');
  }

  if (isProduction) {
    resolvedCompilation.lazyCompilation = false;
  } else if (resolvedCompilation.lazyCompilation === undefined) {
    resolvedCompilation.lazyCompilation ??= isDevelopment;
  }

  resolvedCompilation.mode ??= mode;

  setProcessEnv(resolvedCompilation.mode);

  // TODO add targetEnv `lib-browser` and `lib-node` support
  const is_entry_html =
    !resolvedCompilation.input ||
    Object.values(resolvedCompilation.input).some(
      (value) => value && value.endsWith('.html')
    );

  if (
    resolvedCompilation.output.targetEnv !== 'node' &&
    isArray(resolvedCompilation.runtime.plugins) &&
    resolvedUserConfig.server?.hmr &&
    !resolvedCompilation.runtime.plugins.includes(hmrClientPluginPath)
  ) {
    const publicPath = getValidPublicPath(
      resolvedCompilation.output.publicPath
    );
    const serverOptions = resolvedUserConfig.server;
    const defineHmrPath = normalizePath(
      path.join(publicPath, resolvedUserConfig.server.hmr.path)
    );

    resolvedCompilation.runtime.plugins.push(hmrClientPluginPath);
    // TODO optimize get hmr logic
    resolvedCompilation.define.FARM_HMR_PORT = String(
      (serverOptions.hmr.port || undefined) ??
        serverOptions.port ??
        DEFAULT_DEV_SERVER_OPTIONS.port
    );
    resolvedCompilation.define.FARM_HMR_HOST = JSON.stringify(
      resolvedUserConfig.server.hmr.host
    );
    resolvedCompilation.define.FARM_HMR_PROTOCOL = JSON.stringify(
      resolvedUserConfig.server.hmr.protocol
    );
    resolvedCompilation.define.FARM_HMR_PATH = JSON.stringify(defineHmrPath);
  }

  if (
    isArray(resolvedCompilation.runtime.plugins) &&
    !resolvedCompilation.runtime.plugins.includes(ImportMetaPluginPath)
  ) {
    resolvedCompilation.runtime.plugins.push(ImportMetaPluginPath);
  }

  // we should not deep merge compilation.input
  if (compilation?.input && Object.keys(compilation.input).length > 0) {
    // Add ./ if userConfig.input is relative path without ./
    const input: Record<string, string> = {};

    for (const [key, value] of Object.entries(compilation.input)) {
      if (!value && (value ?? true)) continue;
      if (!path.isAbsolute(value) && !value.startsWith('./')) {
        input[key] = `./${value}`;
      } else {
        input[key] = value;
      }
    }

    resolvedCompilation.input = input;
  }

  if (resolvedCompilation.treeShaking === undefined) {
    resolvedCompilation.treeShaking ??= isProduction;
  }

  if (resolvedCompilation.script?.plugins?.length) {
    resolvedUserConfig.logger.info(
      `Swc plugins are configured, note that Farm uses ${colors.yellow(
        'swc_core v0.96'
      )}, please make sure the plugin is ${colors.green(
        'compatible'
      )} with swc_core ${colors.yellow(
        'swc_core v0.96'
      )}. Otherwise, it may exit unexpectedly.`
    );
  }

  // lazyCompilation should be disabled in production mode
  // so, it only happens in development mode
  // https://github.com/farm-fe/farm/issues/962
  if (resolvedCompilation.treeShaking && resolvedCompilation.lazyCompilation) {
    resolvedUserConfig.logger.error(
      'treeShaking option is not supported in lazyCompilation mode, lazyCompilation will be disabled.'
    );
    resolvedCompilation.lazyCompilation = false;
  }

  if (resolvedCompilation.minify === undefined) {
    resolvedCompilation.minify ??= isProduction;
  }

  if (resolvedCompilation.presetEnv === undefined) {
    resolvedCompilation.presetEnv ??= isProduction;
  }

  // setting the custom configuration
  resolvedCompilation.custom = {
    ...(resolvedCompilation.custom || {}),
    [CUSTOM_KEYS.runtime_isolate]: `${!!resolvedCompilation.runtime.isolate}`
  };

  // Auto enable decorator by default when `script.decorators` is enabled
  if (resolvedCompilation.script?.decorators !== undefined)
    if (resolvedCompilation.script.parser === undefined) {
      resolvedCompilation.script.parser = {
        esConfig: {
          decorators: true
        },
        tsConfig: {
          decorators: true
        }
      };
    } else {
      if (resolvedCompilation.script.parser.esConfig !== undefined)
        resolvedCompilation.script.parser.esConfig.decorators = true;
      else
        resolvedCompilation.script.parser.esConfig = {
          decorators: true
        };
      if (resolvedCompilation.script.parser.tsConfig !== undefined)
        resolvedCompilation.script.parser.tsConfig.decorators = true;
    }

  // normalize persistent cache at last
  await normalizePersistentCache(resolvedCompilation, resolvedUserConfig);
  normalizeResolve(resolvedUserConfig, resolvedCompilation);
  normalizeCss(resolvedUserConfig, resolvedCompilation);
  normalizePartialBundling(resolvedCompilation);
  return resolvedCompilation;
}

export const DEFAULT_HMR_OPTIONS: Required<UserHmrConfig> = {
  host: 'localhost',
  port:
    (process.env.FARM_DEFAULT_HMR_PORT &&
      Number(process.env.FARM_DEFAULT_HMR_PORT)) ??
    undefined,
  path: '/__hmr',
  overlay: true,
  protocol: 'ws',
  watchOptions: {},
  clientPort: 9000,
  timeout: 0,
  server: null,
  channels: []
};

export const DEFAULT_DEV_SERVER_OPTIONS: NormalizedServerConfig = {
  headers: {},
  port:
    (process.env.FARM_DEFAULT_SERVER_PORT &&
      Number(process.env.FARM_DEFAULT_SERVER_PORT)) ||
    9000,
  https: undefined,
  protocol: 'http',
  hostname: { name: 'localhost', host: undefined },
  host: true,
  proxy: undefined,
  hmr: DEFAULT_HMR_OPTIONS,
  middlewareMode: false,
  open: false,
  strictPort: false,
  cors: false,
  middlewares: [],
  appType: 'spa',
  writeToDisk: false
};

export const DEFAULT_COMPILATION_OPTIONS: Partial<ResolvedCompilation> = {
  output: {
    path: './dist'
  },
  sourcemap: true,
  resolve: {
    extensions: [
      'tsx',
      'mts',
      'cts',
      'ts',
      'jsx',
      'mjs',
      'js',
      'cjs',
      'json',
      'html',
      'css',
      'mts',
      'cts'
    ]
  }
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any

function tryHttpsAsFileRead(value: unknown): string | Buffer | unknown {
  if (typeof value === 'string') {
    try {
      const resolvedPath = path.resolve(value);
      const stats = fse.statSync(resolvedPath);

      if (stats.isFile()) {
        return fse.readFileSync(resolvedPath);
      }
    } catch {}
  }

  return Buffer.isBuffer(value) ? value : value;
}

export function normalizeDevServerConfig(
  options: UserServerConfig | undefined,
  mode: string
): NormalizedServerConfig {
  const { host, port, hmr: hmrConfig, https } = options || {};
  const isProductionMode = mode === 'production';
  const hmr =
    isProductionMode || hmrConfig === false
      ? false
      : merge(
          {},
          DEFAULT_HMR_OPTIONS,
          {
            host: host ?? DEFAULT_DEV_SERVER_OPTIONS.host,
            port: port ?? DEFAULT_DEV_SERVER_OPTIONS.port
          },
          hmrConfig === true ? {} : hmrConfig
        );

  return merge({}, DEFAULT_DEV_SERVER_OPTIONS, options, {
    hmr,
    https: https
      ? {
          ...https,
          ca: tryHttpsAsFileRead(options.https.ca),
          cert: tryHttpsAsFileRead(options.https.cert),
          key: tryHttpsAsFileRead(options.https.key),
          pfx: tryHttpsAsFileRead(options.https.pfx)
        }
      : undefined
  }) as NormalizedServerConfig;
}

type Format = Exclude<OutputConfig['format'], undefined>;
const formatFromExt: Record<string, Format> = {
  cjs: 'cjs',
  mjs: 'esm',
  cts: 'cjs',
  mts: 'esm'
};

const formatToExt: Record<Format, string> = {
  cjs: 'cjs',
  esm: 'mjs'
};

export async function readConfigFile(
  inlineOptions: FarmCliOptions,
  configFilePath: string,
  configEnv: any,
  mode: CompilationMode = 'development'
): Promise<UserConfig | undefined> {
  if (!fse.existsSync(configFilePath)) return;

  const format = getFormat(configFilePath);

  const Compiler = (await import('../compiler/index.js')).Compiler;

  const outputPath = path.join(
    path.dirname(configFilePath),
    'node_modules',
    '.farm'
  );

  const fileName = `farm.config.bundle-${Date.now()}-${Math.random()
    .toString(16)
    .split('.')
    .join('')}.${formatToExt[format]}`;

  const normalizedConfig = await resolveDefaultUserConfig({
    inlineOptions,
    configFilePath,
    format,
    outputPath,
    fileName,
    mode
  });

  const replaceDirnamePlugin = await import(
    '@farmfe/plugin-replace-dirname'
  ).then((mod) => mod.default);

  const compiler = new Compiler({
    compilation: normalizedConfig,
    jsPlugins: [],
    rustPlugins: [[replaceDirnamePlugin, '{}']]
  });

  const FARM_PROFILE = process.env.FARM_PROFILE;
  // disable FARM_PROFILE in farm_config
  if (FARM_PROFILE) {
    process.env.FARM_PROFILE = '';
  }

  try {
    await compiler.compile();

    if (FARM_PROFILE) {
      process.env.FARM_PROFILE = FARM_PROFILE;
    }

    compiler.writeResourcesToDisk();

    const filePath = getFilePath(outputPath, fileName);

    // Change to vm.module of node or loaders as far as it is stable
    const userConfig = (await import(filePath as string)).default;
    try {
      fs.unlink(filePath, () => void 0);
      // remove parent dir if empty
      const isEmpty = fs.readdirSync(outputPath).length === 0;
      if (isEmpty) {
        fs.rmSync(outputPath);
      }
    } catch {
      /** do nothing */
    }

    const config = await (typeof userConfig === 'function'
      ? userConfig(configEnv)
      : userConfig);

    if (!isObject(config)) {
      throw new Error(`config must export or return an object.`);
    }

    config.root ??= inlineOptions.root;

    return config;
  } finally {
    fse.unlink(getFilePath(outputPath, fileName)).catch(() => {});
  }
}

export function normalizePublicDir(root: string, publicDir = 'public') {
  const absPublicDirPath = path.isAbsolute(publicDir)
    ? publicDir
    : path.resolve(root, publicDir);

  return absPublicDirPath;
}

/**
 * Load config file from the specified path and return the config and config file path
 * @param configPath the config path, could be a directory or a file
 * @param logger custom logger
 * @returns loaded config and config file path
 */
export async function loadConfigFile(
  configFile: string,
  inlineOptions: any,
  configEnv: any,
  mode: CompilationMode = 'development'
): Promise<{ config: any; configFilePath: string } | undefined> {
  const { root = '.' } = inlineOptions;
  const configRootPath = path.resolve(root);
  let resolvedPath: string | undefined;
  try {
    resolvedPath = await resolveConfigFilePath(
      configFile,
      root,
      configRootPath
    );

    const config = await readConfigFile(
      inlineOptions,
      resolvedPath,
      configEnv,
      mode
    );
    return {
      config: config && parseUserConfig(config),
      configFilePath: resolvedPath
    };
  } catch (error) {
    // In this place, the original use of throw caused emit to the outermost catch
    // callback, causing the code not to execute. If the internal catch compiler's own
    // throw error can solve this problem, it will not continue to affect the execution of
    // external code. We just need to return the default config.
    const errorMessage = convertErrorMessage(error);
    const stackTrace =
      error.code === 'GenericFailure' ? '' : `\n${error.stack}`;
    if (inlineOptions.mode === 'production') {
      throw new Error(
        `Failed to load farm config file: ${errorMessage} \n${stackTrace}`
      );
    }
    const potentialSolution =
      'Potential solutions: \n1. Try set `FARM_CONFIG_FORMAT=cjs`(default to esm)\n2. Try set `FARM_CONFIG_FULL_BUNDLE=1`';
    // throw new Error(
    // `Failed to load farm config file: ${errorMessage}. \n ${potentialSolution} \n ${error.stack}`
    // );
    throw new Error(
      `Failed to load farm config file: ${errorMessage}. \n ${potentialSolution}`
      // `Failed to load farm config file: ${errorMessage}.`,
    );
  }
}

export async function checkCompilationInputValue(
  userConfig: UserConfig,
  logger: Logger
) {
  const { compilation } = userConfig;
  const targetEnv = compilation?.output?.targetEnv;
  const inputValue = Object.values(compilation?.input).filter(Boolean);
  const isTargetNode = targetEnv === 'node';
  const defaultHtmlPath = './index.html';
  let inputIndexConfig: { index?: string } = { index: '' };
  let errorMessage = '';

  // Check if input is specified
  if (!isEmptyObject(compilation?.input) && inputValue.length) {
    inputIndexConfig = compilation?.input;
  } else {
    const rootPath = userConfig?.root ?? '.';
    if (isTargetNode) {
      // If input is not specified, try to find index.js or index.ts
      const entryFiles = ['./index.js', './index.ts'];

      for (const entryFile of entryFiles) {
        try {
          const resolvedPath = path.resolve(rootPath, entryFile);
          if (await checkFileExists(resolvedPath)) {
            inputIndexConfig = { index: entryFile };
            break;
          }
        } catch (error) {
          errorMessage = error.stack;
        }
      }
    } else {
      try {
        const resolvedHtmlPath = path.resolve(rootPath, defaultHtmlPath);
        if (await checkFileExists(resolvedHtmlPath)) {
          inputIndexConfig = { index: defaultHtmlPath };
        }
      } catch (error) {
        errorMessage = error.stack;
      }
    }

    // If no index file is found, throw an error
    if (!inputIndexConfig.index) {
      logger.error(
        `Build failed due to errors: Can not resolve ${
          isTargetNode ? 'index.js or index.ts' : 'index.html'
        }  from ${userConfig.root}. \n${errorMessage}`
        // { exit: true }
      );
    }
  }

  return inputIndexConfig;
}

export async function getConfigFilePath(
  configRootPath: string
): Promise<string | undefined> {
  const stat = await fse.stat(configRootPath);
  if (!stat.isDirectory()) {
    return undefined;
  }

  for (const name of DEFAULT_CONFIG_NAMES) {
    const resolvedPath = path.join(configRootPath, name);
    try {
      const fileStat = await fse.stat(resolvedPath);
      if (fileStat.isFile()) {
        return resolvedPath;
      }
    } catch {}
  }

  return undefined;
}

export async function resolvePlugins(
  userConfig: UserConfig,
  mode: CompilationMode
) {
  const { jsPlugins: rawJsPlugins, rustPlugins } =
    await resolveFarmPlugins(userConfig);
  const jsPlugins = await resolveAndFilterAsyncPlugins(rawJsPlugins);

  const vitePlugins = (userConfig?.vitePlugins ?? []).filter(Boolean);

  const vitePluginAdapters = vitePlugins.length
    ? await handleVitePlugins(vitePlugins, userConfig, mode)
    : [];

  return {
    jsPlugins,
    vitePlugins,
    rustPlugins,
    vitePluginAdapters
  };
}

export async function resolveDefaultUserConfig(options: any) {
  const defaultConfig: UserConfig = createDefaultConfig(options);

  const resolvedUserConfig: ResolvedUserConfig = await resolveUserConfig(
    defaultConfig,
    undefined,
    defaultConfig.compilation.mode
  );

  const normalizedConfig = await normalizeUserCompilationConfig(
    resolvedUserConfig,
    'development'
  );

  return normalizedConfig;
}

export async function resolveUserConfig(
  userConfig: UserConfig,
  configFilePath?: string | undefined,
  mode: 'development' | 'production' | string = 'development'
): Promise<ResolvedUserConfig> {
  const resolvedUserConfig = {
    ...userConfig,
    envMode: mode
  } as ResolvedUserConfig;

  // set internal config
  if (configFilePath) {
    const dependencies = await traceDependencies(configFilePath);
    resolvedUserConfig.configFileDependencies = dependencies.sort();
    resolvedUserConfig.configFilePath = configFilePath;
  }

  const resolvedRootPath = resolvedUserConfig.root;
  const resolvedEnvPath = resolvedUserConfig.envDir ?? resolvedRootPath;

  const userEnv = loadEnv(
    resolvedUserConfig.envMode,
    resolvedEnvPath,
    resolvedUserConfig.envPrefix
  );
  const existsEnvFiles = getExistsEnvFiles(
    resolvedUserConfig.envMode,
    resolvedEnvPath
  );

  resolvedUserConfig.envFiles = [
    ...(Array.isArray(resolvedUserConfig.envFiles)
      ? resolvedUserConfig.envFiles
      : []),
    ...existsEnvFiles
  ];

  resolvedUserConfig.env = {
    ...userEnv,
    NODE_ENV: userConfig.compilation.mode,
    // TODO publicPath rewrite to BASE_URL
    BASE_URL: userConfig.compilation.output.publicPath ?? '/',
    mode,
    DEV: mode === 'development',
    PROD: mode === 'production'
  };

  resolvedUserConfig.publicDir = normalizePublicDir(
    resolvedRootPath,
    userConfig.publicDir
  );

  // TODO type error
  // @ts-ignore
  // resolveUserConfig.logger = logger;

  return resolvedUserConfig;
}

export function createDefaultConfig(options: any): UserConfig {
  const { inlineOptions, mode, format, outputPath, fileName, configFilePath } =
    options;

  return {
    root: path.resolve(inlineOptions.root ?? '.'),
    compilation: {
      input: {
        [fileName]: configFilePath
      },
      output: {
        entryFilename: '[entryName]',
        path: outputPath,
        format,
        targetEnv: 'library-node'
      },
      mode,
      external: [
        ...(process.env.FARM_CONFIG_FULL_BUNDLE
          ? []
          : ['!^(\\./|\\.\\./|[A-Za-z]:\\\\|/).*']),
        '^@farmfe/core$'
      ],
      partialBundling: {
        enforceResources: [
          {
            name: fileName,
            test: ['.+']
          }
        ]
      },
      sourcemap: false,
      treeShaking: false,
      minify: false,
      presetEnv: false,
      lazyCompilation: false,
      persistentCache: false,
      progress: false
    }
  };
}

export async function resolveAndFilterAsyncPlugins(
  plugins: JsPlugin[] = []
): Promise<JsPlugin[]> {
  return (await resolveAsyncPlugins(plugins)).filter(Boolean);
}

export async function checkFileExists(filePath: string): Promise<boolean> {
  try {
    await fse.stat(filePath);
    return true;
  } catch {
    return false;
  }
}

export async function resolveConfigFilePath(
  configFile: string | undefined,
  root: string,
  configRootPath: string
): Promise<string | undefined> {
  if (configFile) {
    return path.resolve(root, configFile);
  } else {
    return await getConfigFilePath(configRootPath);
  }
}

// export async function handleServerPortConflict(
//   resolvedUserConfig: ResolvedUserConfig,
//   logger: Logger,
//   mode?: CompilationMode
// ) {
//   // check port availability: auto increment the port if a conflict occurs

//   try {
//     mode !== 'production' &&
//       (await Server.resolvePortConflict(resolvedUserConfig.server, logger));
//     // eslint-disable-next-line no-empty
//   } catch {}
// }

export function checkClearScreen(
  inlineConfig: FarmCliOptions | ResolvedUserConfig
) {
  if (
    inlineConfig?.clearScreen &&
    !__FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__
  ) {
    clearScreen();
  }
}

export function getFormat(configFilePath: string): Format {
  return process.env.FARM_CONFIG_FORMAT === 'cjs'
    ? 'cjs'
    : process.env.FARM_CONFIG_FORMAT === 'esm'
      ? 'esm'
      : formatFromExt[path.extname(configFilePath).slice(1)] ?? 'esm';
}

export function getFilePath(outputPath: string, fileName: string): string {
  return isWindows
    ? pathToFileURL(path.join(outputPath, fileName)).toString()
    : path.join(outputPath, fileName);
}

async function setLazyCompilationDefine(
  resolvedUserConfig: ResolvedUserConfig
) {
  const hostname = await resolveHostname(resolvedUserConfig.server.host);
  resolvedUserConfig.compilation.define = {
    ...(resolvedUserConfig.compilation.define ?? {}),
    FARM_LAZY_COMPILE_SERVER_URL: `${
      resolvedUserConfig.server.protocol || 'http'
    }://${hostname.host || 'localhost'}:${resolvedUserConfig.server.port}`
  };
}
