import { createHash } from 'node:crypto';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

import fse from 'fs-extra';

import { bindingPath } from '../../binding/index.js';
import { JsPlugin } from '../index.js';
import {
  type RustPlugin,
  getSortedPlugins,
  resolveAsyncPlugins,
  resolveConfigHook,
  resolveConfigResolvedHook,
  resolveFarmPlugins,
  resolveVitePlugins
} from '../plugin/index.js';

import {
  Logger,
  clearScreen,
  colors,
  isArray,
  isEmptyObject,
  isNodeEnv,
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
  ENV_DEVELOPMENT,
  ENV_PRODUCTION,
  FARM_DEFAULT_NAMESPACE
} from './constants.js';
import { mergeConfig, mergeFarmCliConfig } from './mergeConfig.js';

import { normalizeCss } from './normalize-config/normalize-css.js';
import { normalizeExternal } from './normalize-config/normalize-external.js';
import { normalizePartialBundling } from './normalize-config/normalize-partial-bundling.js';
import { normalizeResolve } from './normalize-config/normalize-resolve.js';

import { wrapPluginUpdateModules } from '../plugin/js/utils.js';
import type {
  ConfigEnv,
  ConfigResult,
  DefaultOptionsType,
  EnvResult,
  FarmCliOptions,
  Format,
  HmrOptions,
  NormalizedServerConfig,
  ResolvedCompilation,
  ResolvedUserConfig,
  UserConfig,
  UserConfigExport,
  UserConfigFnObject,
  commandType
} from './types.js';

export * from './types.js';
export * from './constants.js';
export * from './env.js';

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

type UserConfigPromise = Promise<UserConfig | undefined>;

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
  command: commandType,
  defaultMode: CompilationMode = 'development',
  defaultNodeEnv: CompilationMode = 'development',
  isPreview = false
): Promise<ResolvedUserConfig> {
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

  let configFilePath;

  const loadedUserConfig = await loadConfigFile(
    inlineOptions,
    configEnv,
    defaultNodeEnv
  );

  let userConfig: UserConfig = mergeFarmCliConfig(
    inlineOptions,
    {},
    defaultMode
  );

  const transformInlineConfig = userConfig;

  if (loadedUserConfig) {
    configFilePath = loadedUserConfig.configFilePath;
    userConfig = mergeConfig(userConfig, loadedUserConfig.config);
  }

  const { jsPlugins, vitePluginAdapters } = await resolvePlugins(
    userConfig,
    defaultMode
  );
  const sortFarmJsPlugins = getSortedPlugins([
    ...jsPlugins,
    ...vitePluginAdapters
  ]);

  const config = await resolveConfigHook(userConfig, sortFarmJsPlugins);
  // may be user push plugin when config hooks
  const allPlugins = await resolvePlugins(config, defaultMode);
  const farmJsPlugins = getSortedPlugins([
    ...allPlugins.jsPlugins,
    ...vitePluginAdapters,
    externalAdapter()
  ]);

  const resolvedUserConfig = await handleResolveConfig(
    configFilePath,
    loadedUserConfig,
    config,
    farmJsPlugins,
    allPlugins.rustPlugins,
    transformInlineConfig,
    command
  );

  await resolveConfigResolvedHook(resolvedUserConfig, sortFarmJsPlugins); // Fix: Await the Promise<void> and pass the resolved value to the function.

  return resolvedUserConfig;
}

async function handleResolveConfig(
  configFilePath: string,
  loadedUserConfig:
    | {
        config: UserConfig;
        configFilePath: string;
      }
    | undefined,
  config: UserConfig,
  sortFarmJsPlugins: JsPlugin[],
  rustPlugins: RustPlugin[],
  transformInlineConfig: UserConfig,
  command: commandType
): Promise<ResolvedUserConfig> {
  // define logger when resolvedConfigHook
  const logger = new Logger({
    customLogger: loadedUserConfig.config?.customLogger,
    allowClearScreen: loadedUserConfig.config?.clearScreen
  });

  const resolvedUserConfig = await resolveUserConfig(config, configFilePath);

  resolvedUserConfig.logger = logger;

  // farm handles server attributes in resolveConfig.
  // On the one hand, farm can be used in node and server needs
  // to be enabled. Lazy loading mode is enabled in node environment.
  resolvedUserConfig.server = normalizeDevServerConfig(resolvedUserConfig);

  resolvedUserConfig.compilation =
    await normalizeUserCompilationConfig(resolvedUserConfig);

  Object.assign(resolvedUserConfig, {
    root: resolvedUserConfig.compilation.root,
    jsPlugins: sortFarmJsPlugins,
    rustPlugins: rustPlugins,
    command,
    isProduction: resolvedUserConfig.compilation.mode === ENV_PRODUCTION,
    transformInlineConfig
  });

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
    [COMMANDS.START]: async (config: ResolvedUserConfig) => {
      if (
        config.compilation.lazyCompilation &&
        typeof config.server?.host === 'string'
      ) {
        await setLazyCompilationDefine(config);
      }
    },
    // TODO 这个watch 方法需要在讨论 现在设计里没有 watch 这个方法了 build 的话也可以做 判断 config 里的 watch
    [COMMANDS.WATCH]: async (config: ResolvedUserConfig) => {
      if (config.compilation?.lazyCompilation) {
        await setLazyCompilationDefine(config);
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
  const inputIndexConfig = await checkCompilationInputValue(resolvedUserConfig);

  const resolvedCompilation: ResolvedCompilation = merge(
    {},
    DEFAULT_COMPILATION_OPTIONS,
    {
      input: inputIndexConfig,
      root: resolvedRootPath
    },
    compilation
  );

  const isProduction = mode === ENV_PRODUCTION;
  const isDevelopment = mode === ENV_DEVELOPMENT;
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
      : Object.keys(resolvedUserConfig.env || {}).reduce<EnvResult>(
          (env, key) => {
            env[`$__farm_regex:(global(This)?\\.)?process\\.env\\.${key}`] =
              JSON.stringify(resolvedUserConfig.env[key]);
            return env;
          },
          {} as EnvResult
        )
  );

  const require = createRequire(import.meta.url);
  const hmrClientPluginPath = require.resolve('@farmfe/runtime-plugin-hmr');
  const importMetaPluginPath = require.resolve(
    '@farmfe/runtime-plugin-import-meta'
  );

  resolvedCompilation.runtime = {
    path:
      resolvedCompilation.runtime?.path ?? require.resolve('@farmfe/runtime'),
    swcHelpersPath:
      resolvedCompilation.runtime?.swcHelpersPath ??
      path.dirname(require.resolve('@swc/helpers/package.json')),
    plugins: resolvedCompilation.runtime?.plugins ?? [],
    namespace: resolvedCompilation.runtime?.namespace
  };

  resolvedCompilation.runtime.plugins = resolvedCompilation.runtime.plugins.map(
    (plugin) => {
      if (path.isAbsolute(plugin)) return plugin;
      return plugin.startsWith('.')
        ? path.resolve(resolvedRootPath, plugin)
        : require.resolve(plugin);
    }
  );

  if (!resolvedCompilation.runtime.namespace) {
    resolvedCompilation.runtime.namespace = createHash('md5')
      .update(getNamespaceName(resolvedRootPath))
      .digest('hex');
  }

  if (isProduction) {
    resolvedCompilation.lazyCompilation = false;
  } else if (resolvedCompilation.lazyCompilation === undefined) {
    resolvedCompilation.lazyCompilation ??= isDevelopment;
  }

  resolvedCompilation.mode ??= mode;

  setProcessEnv(resolvedCompilation.mode);
  const isNode = isNodeEnv(resolvedCompilation.output.targetEnv);
  if (
    !isNode &&
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
    !resolvedCompilation.runtime.plugins.includes(importMetaPluginPath)
  ) {
    resolvedCompilation.runtime.plugins.push(importMetaPluginPath);
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

  if (resolvedCompilation.concatenateModules === undefined) {
    resolvedCompilation.concatenateModules ??= isProduction;
  }

  if (resolvedCompilation.concatenateModules && !isProduction) {
    resolvedUserConfig.logger.warn(
      'concatenateModules option is not supported with development mode, concatenateModules will be disabled'
    );
    resolvedCompilation.concatenateModules = false;
  }

  if (resolvedCompilation.script?.plugins?.length) {
    resolvedUserConfig.logger.info(
      `Swc plugins are configured, note that Farm uses ${colors.yellow(
        'swc_core v0.96'
      )}, please make sure the plugin is ${colors.green('compatible')} with swc_core ${colors.yellow(
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

export const DEFAULT_HMR_OPTIONS: Required<HmrOptions> = {
  host: 'localhost',
  port:
    (process.env.FARM_DEFAULT_HMR_PORT &&
      Number(process.env.FARM_DEFAULT_HMR_PORT)) ??
    undefined,
  path: '/__hmr',
  overlay: true,
  clientPort: 9000,
  timeout: 0,
  server: null,
  protocol: ''
};

export const DEFAULT_DEV_SERVER_OPTIONS: NormalizedServerConfig = {
  headers: {},
  port:
    (process.env.FARM_DEFAULT_SERVER_PORT &&
      Number(process.env.FARM_DEFAULT_SERVER_PORT)) ||
    9000,
  https: undefined,
  protocol: 'http',
  hostname: {
    name: 'localhost',
    host: undefined
  },
  host: 'localhost',
  proxy: undefined,
  hmr: DEFAULT_HMR_OPTIONS,
  middlewareMode: false,
  open: false,
  strictPort: false,
  cors: false,
  middlewares: [],
  appType: 'spa',
  writeToDisk: false,
  origin: '',
  preview: {
    host: 'localhost',
    headers: {},
    port: 1911,
    strictPort: false,
    https: undefined,
    distDir: 'dist',
    open: false,
    cors: false,
    proxy: undefined
  }
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
  userConfig: UserConfig | undefined
): NormalizedServerConfig {
  const serverOptions = userConfig?.server;
  const { host, port, hmr: hmrConfig, https } = serverOptions || {};
  const hmr =
    hmrConfig === false
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

  return merge({}, DEFAULT_DEV_SERVER_OPTIONS, serverOptions, {
    hmr,
    https: https
      ? {
          ...https,
          ca: tryHttpsAsFileRead(serverOptions.https.ca),
          cert: tryHttpsAsFileRead(serverOptions.https.cert),
          key: tryHttpsAsFileRead(serverOptions.https.key),
          pfx: tryHttpsAsFileRead(serverOptions.https.pfx)
        }
      : undefined
  }) as NormalizedServerConfig;
}

const formatFromExt: Record<string, Format> = {
  cjs: 'cjs',
  mjs: 'esm',
  cts: 'cjs',
  mts: 'esm',
  js: 'esm'
};

const formatToExt: Record<Format, string> = {
  cjs: 'cjs',
  esm: 'mjs'
};

export async function readConfigFile(
  inlineOptions: FarmCliOptions,
  configFilePath: string,
  configEnv: ConfigEnv,
  mode: CompilationMode = 'development'
): UserConfigPromise {
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
  inlineOptions: FarmCliOptions & UserConfig,
  configEnv: ConfigEnv,
  mode: CompilationMode = 'development'
): Promise<ConfigResult | undefined> {
  const { root = '.', configFile } = inlineOptions;
  const configRootPath = path.resolve(root);
  let resolvedConfigFilePath: string | undefined;
  try {
    resolvedConfigFilePath = await resolveConfigFilePath(
      configFile,
      root,
      configRootPath
    );

    const config = await readConfigFile(
      inlineOptions,
      resolvedConfigFilePath,
      configEnv,
      mode
    );
    return {
      config: config && parseUserConfig(config),
      configFilePath: resolvedConfigFilePath
    };
  } catch (error) {
    // In this place, the original use of throw caused emit to the outermost catch
    // callback, causing the code not to execute. If the internal catch compiler's own
    // throw error can solve this problem, it will not continue to affect the execution of
    // external code. We just need to return the default config.
    const errorMessage = convertErrorMessage(error);
    const stackTrace =
      error.code === 'GenericFailure' ? '' : `\n${error.stack}`;
    if (inlineOptions.mode === ENV_PRODUCTION) {
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
  userConfig: ResolvedUserConfig
) {
  const { compilation } = userConfig;
  const targetEnv = compilation?.output?.targetEnv;
  const inputValue = Object.values(compilation?.input).filter(Boolean);
  const isTargetNode = isNodeEnv(targetEnv);
  const defaultHtmlPath = './index.html';
  let inputIndexConfig: {
    index?: string;
  } = { index: '' };
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
            inputIndexConfig = {
              index: entryFile
            };
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
          inputIndexConfig = {
            index: defaultHtmlPath
          };
        }
      } catch (error) {
        errorMessage = error.stack;
      }
    }

    // If no index file is found, throw an error
    if (!inputIndexConfig.index) {
      userConfig.logger.error(
        `Build failed due to errors: Can not resolve ${
          isTargetNode ? 'index.js or index.ts' : 'index.html'
        }  from ${userConfig.root}. \n${errorMessage}`
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
  const [farmPlugins, vitePluginAdapters] = await Promise.all([
    resolveFarmPlugins(userConfig),
    resolveVitePlugins(userConfig, mode)
  ]);

  const resolvePluginsResult = {
    jsPlugins: farmPlugins.jsPlugins.map(wrapPluginUpdateModules),
    vitePlugins: (userConfig?.vitePlugins ?? []).filter(Boolean),
    rustPlugins: farmPlugins.rustPlugins,
    vitePluginAdapters
  };

  return resolvePluginsResult;
}

export async function resolveDefaultUserConfig(options: DefaultOptionsType) {
  const defaultConfig: UserConfig = createDefaultConfig(options);

  const resolvedUserConfig: ResolvedUserConfig = await resolveUserConfig(
    defaultConfig,
    undefined
  );

  const normalizedConfig = await normalizeUserCompilationConfig(
    resolvedUserConfig,
    options.mode
  );

  return normalizedConfig;
}

export async function resolveUserConfig(
  userConfig: UserConfig,
  configFilePath?: string | undefined
): Promise<ResolvedUserConfig> {
  const resolvedUserConfig = {
    ...userConfig,
    envMode: userConfig.mode
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
    BASE_URL: userConfig.compilation.output.publicPath ?? '/',
    mode: userConfig.mode,
    DEV: userConfig.compilation.mode === ENV_DEVELOPMENT,
    PROD: userConfig.compilation.mode === ENV_PRODUCTION
  };

  resolvedUserConfig.publicDir = normalizePublicDir(
    resolvedRootPath,
    userConfig.publicDir
  );

  return resolvedUserConfig;
}

export function createDefaultConfig(options: DefaultOptionsType): UserConfig {
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

export function getFormat(configFilePath: string): Format {
  return process.env.FARM_CONFIG_FORMAT === 'cjs'
    ? 'cjs'
    : process.env.FARM_CONFIG_FORMAT === 'esm'
      ? 'esm'
      : (formatFromExt[path.extname(configFilePath).slice(1)] ?? 'esm');
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

function getNamespaceName(rootPath: string) {
  const packageJsonPath = path.resolve(rootPath, 'package.json');
  if (fse.existsSync(packageJsonPath)) {
    const { name } = JSON.parse(fse.readFileSync(packageJsonPath, 'utf-8'));
    return name || FARM_DEFAULT_NAMESPACE;
  }
  return FARM_DEFAULT_NAMESPACE;
}
