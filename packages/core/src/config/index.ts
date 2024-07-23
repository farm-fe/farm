import crypto from 'node:crypto';
import fs from 'node:fs';
import module from 'node:module';
import path, { isAbsolute, join } from 'node:path';
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
  resolveFarmPlugins,
  rustPluginResolver
} from '../plugin/index.js';
import { Server } from '../server/index.js';
import {
  Logger,
  bold,
  clearScreen,
  colors,
  getAliasEntries,
  green,
  isArray,
  isEmptyObject,
  isObject,
  isWindows,
  normalizeBasePath,
  normalizePath,
  transformAliasWithVite
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
import merge from '../utils/merge.js';
import {
  CUSTOM_KEYS,
  DEFAULT_CONFIG_NAMES,
  FARM_DEFAULT_NAMESPACE
} from './constants.js';
import { mergeConfig, mergeFarmCliConfig } from './mergeConfig.js';
import { normalizeExternal } from './normalize-config/normalize-external.js';
import type {
  Alias,
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

export async function handleServerPortConflict(
  resolvedUserConfig: ResolvedUserConfig,
  logger: Logger,
  mode?: CompilationMode
) {
  // check port availability: auto increment the port if a conflict occurs

  try {
    mode !== 'production' &&
      (await Server.resolvePortConflict(resolvedUserConfig.server, logger));
    // eslint-disable-next-line no-empty
  } catch {}
}

/**
 * Resolve and load user config from the specified path
 * @param configPath
 */
export async function resolveConfig(
  inlineOptions: FarmCliOptions & UserConfig,
  command: 'start' | 'build' | 'preview',
  defaultMode: CompilationMode = 'development',
  defaultNodeEnv: CompilationMode = 'development',
  isPreview = false,
  logger?: Logger
): Promise<ResolvedUserConfig> {
  logger = logger ?? new Logger();

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
  const loadedUserConfig: any = await loadConfigFile(
    configFile,
    inlineOptions,
    configEnv
  );

  let rawConfig: UserConfig = mergeFarmCliConfig(
    inlineOptions,
    {},
    compileMode
  );
  let configPath = initialConfigPath;

  if (loadedUserConfig) {
    configPath = loadedUserConfig.configFilePath;
    rawConfig = mergeConfig(rawConfig, loadedUserConfig.config);
  }

  const { config: userConfig, configFilePath } = {
    configFilePath: configPath,
    config: rawConfig
  };

  const { jsPlugins, vitePlugins, rustPlugins, vitePluginAdapters } =
    await resolvePlugins(userConfig, logger, compileMode);

  const sortFarmJsPlugins = getSortedPlugins([
    ...jsPlugins,
    ...vitePluginAdapters,
    externalAdapter()
  ]);

  const config = await resolveConfigHook(userConfig, sortFarmJsPlugins);

  const resolvedUserConfig = await resolveUserConfig(
    config,
    configFilePath,
    compileMode,
    logger
  );

  // normalize server config first cause it may be used in normalizeUserCompilationFnConfig
  resolvedUserConfig.server = normalizeDevServerConfig(
    resolvedUserConfig.server,
    compileMode
  );

  // if (isHandleServerPortConflict) {
  //   await handleServerPortConflict(resolvedUserConfig, logger, mode);
  // }

  resolvedUserConfig.compilation = await normalizeUserCompilationConfig(
    resolvedUserConfig,
    'development'
  );

  resolvedUserConfig.root = resolvedUserConfig.compilation.root;
  resolvedUserConfig.jsPlugins = sortFarmJsPlugins;
  resolvedUserConfig.rustPlugins = rustPlugins;

  // Temporarily dealing with alias objects and arrays in js will be unified in rust in the future.]
  if (vitePlugins.length) {
    resolvedUserConfig.compilation.resolve.alias = getAliasEntries(
      resolvedUserConfig.compilation.resolve.alias
    );
  }

  await resolveConfigResolvedHook(resolvedUserConfig, sortFarmJsPlugins); // Fix: Await the Promise<void> and pass the resolved value to the function.

  //TODO Temporarily solve the problem of alias adaptation to vite
  if (resolvedUserConfig.compilation?.resolve?.alias && vitePlugins.length) {
    resolvedUserConfig.compilation.resolve.alias = transformAliasWithVite(
      resolvedUserConfig.compilation.resolve.alias as unknown as Array<Alias>
    );
  }

  return resolvedUserConfig;
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
  mode: CompilationMode = 'development',
  logger: Logger = new Logger()
): Promise<ResolvedCompilation> {
  const { compilation, root, clearScreen } = resolvedUserConfig;

  // resolve root path

  const resolvedRootPath = normalizePath(root);

  resolvedUserConfig.root = resolvedRootPath;

  // if normalize default config, skip check input option
  const inputIndexConfig = checkCompilationInputValue(
    resolvedUserConfig,
    logger
  );

  const resolvedCompilation: ResolvedCompilation = merge(
    {},
    DEFAULT_COMPILATION_OPTIONS,
    {
      input: inputIndexConfig,
      root: resolvedRootPath
    },
    {
      clearScreen
    },
    compilation
  );

  const isProduction = mode === 'production';
  const isDevelopment = mode === 'development';
  resolvedCompilation.mode = resolvedCompilation.mode ?? mode;

  resolvedCompilation.coreLibPath = bindingPath;

  normalizeOutput(resolvedCompilation, isProduction, logger);
  normalizeExternal(resolvedUserConfig, resolvedCompilation);

  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore do not check type for this internal option
  if (!resolvedCompilation.assets?.publicDir) {
    if (!resolvedCompilation.assets) {
      resolvedCompilation.assets = {};
    }

    const userPublicDir = resolvedUserConfig.publicDir
      ? resolvedUserConfig.publicDir
      : join(resolvedCompilation.root, 'public');

    if (isAbsolute(userPublicDir)) {
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore do not check type for this internal option
      resolvedCompilation.assets.publicDir = userPublicDir;
    } else {
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore do not check type for this internal option
      resolvedCompilation.assets.publicDir = join(
        resolvedCompilation.root,
        userPublicDir
      );
    }
  }

  resolvedCompilation.define = Object.assign(
    {
      // skip self define
      ['FARM' + '_PROCESS_ENV']: resolvedUserConfig.env
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

  const require = module.createRequire(import.meta.url);
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
    // make sure all plugin paths are absolute
    resolvedCompilation.runtime.plugins =
      resolvedCompilation.runtime.plugins.map((plugin) => {
        if (!path.isAbsolute(plugin)) {
          if (!plugin.startsWith('.')) {
            // resolve plugin from node_modules
            return require.resolve(plugin);
          } else {
            return path.resolve(resolvedRootPath, plugin);
          }
        }

        return plugin;
      });
  }
  // set namespace to package.json name field's hash
  if (!resolvedCompilation.runtime.namespace) {
    // read package.json name field
    const packageJsonPath = path.resolve(resolvedRootPath, 'package.json');
    const packageJsonExists = fse.existsSync(packageJsonPath);
    const namespaceName = packageJsonExists
      ? JSON.parse(fse.readFileSync(packageJsonPath, { encoding: 'utf-8' }))
          ?.name ?? FARM_DEFAULT_NAMESPACE
      : FARM_DEFAULT_NAMESPACE;

    resolvedCompilation.runtime.namespace = crypto
      .createHash('md5')
      .update(namespaceName)
      .digest('hex');
  }

  if (isProduction) {
    resolvedCompilation.lazyCompilation = false;
  } else if (resolvedCompilation.lazyCompilation === undefined) {
    if (isDevelopment) {
      resolvedCompilation.lazyCompilation = true;
    } else {
      resolvedCompilation.lazyCompilation = false;
    }
  }

  if (resolvedCompilation.mode === undefined) {
    resolvedCompilation.mode = mode;
  }
  setProcessEnv(resolvedCompilation.mode);

  // TODO add targetEnv `lib-browser` and `lib-node` support
  const is_entry_html =
    Object.keys(resolvedCompilation.input).length === 0 ||
    Object.values(resolvedCompilation.input)
      .filter(Boolean)
      .some((value) => value.endsWith('.html'));

  if (
    resolvedCompilation.output.targetEnv !== 'node' &&
    isArray(resolvedCompilation.runtime.plugins) &&
    resolvedUserConfig.server?.hmr &&
    is_entry_html &&
    !resolvedCompilation.runtime.plugins.includes(hmrClientPluginPath)
  ) {
    const publicPath = getValidPublicPath(
      resolvedCompilation.output.publicPath
    );
    const serverOptions = resolvedUserConfig.server;
    const defineHmrPath = normalizeBasePath(
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
    if (isProduction) {
      resolvedCompilation.treeShaking = true;
    } else {
      resolvedCompilation.treeShaking = false;
    }
  }

  if (resolvedCompilation.script?.plugins?.length) {
    logger.info(
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
    logger.error(
      'treeShaking option is not supported in lazyCompilation mode, lazyCompilation will be disabled.'
    );
    resolvedCompilation.lazyCompilation = false;
  }

  if (resolvedCompilation.minify === undefined) {
    if (isProduction) {
      resolvedCompilation.minify = true;
    } else {
      resolvedCompilation.minify = false;
    }
  }

  if (resolvedCompilation.presetEnv === undefined) {
    if (isProduction) {
      resolvedCompilation.presetEnv = true;
    } else {
      resolvedCompilation.presetEnv = false;
    }
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
  await normalizePersistentCache(
    resolvedCompilation,
    resolvedUserConfig,
    logger
  );

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
  proxy: {},
  hmr: DEFAULT_HMR_OPTIONS,
  middlewareMode: false,
  open: false,
  strictPort: false,
  cors: false,
  spa: true,
  middlewares: [],
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
function tryAsFileRead(value?: any): string | Buffer {
  if (typeof value === 'string' && fs.existsSync(value)) {
    return fs.readFileSync(path.resolve(value.toString()));
  }

  return value;
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
          ca: tryAsFileRead(options.https.ca),
          cert: tryAsFileRead(options.https.cert),
          key: tryAsFileRead(options.https.key),
          pfx: tryAsFileRead(options.https.pfx)
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
  logger: Logger
): Promise<UserConfig | undefined> {
  if (fse.existsSync(configFilePath)) {
    !__FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ &&
      logger.info(`Using config file at ${bold(green(configFilePath))}`);
    const format: Format = process.env.FARM_CONFIG_FORMAT
      ? process.env.FARM_CONFIG_FORMAT === 'cjs'
        ? 'cjs'
        : 'esm'
      : formatFromExt[path.extname(configFilePath).slice(1)] ?? 'esm';

    // we need transform all type farm.config with __dirname and __filename
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
      fileName
    });

    const replaceDirnamePlugin = await rustPluginResolver(
      'farm-plugin-replace-dirname',
      // normalizedConfig.root!,
      process.cwd()
    );

    const compiler = new Compiler(
      {
        config: normalizedConfig,
        jsPlugins: [],
        rustPlugins: [replaceDirnamePlugin]
      },
      logger
    );

    const FARM_PROFILE = process.env.FARM_PROFILE;
    // disable FARM_PROFILE in farm_config
    if (FARM_PROFILE) {
      process.env.FARM_PROFILE = '';
    }
    await compiler.compile();

    if (FARM_PROFILE) {
      process.env.FARM_PROFILE = FARM_PROFILE;
    }

    compiler.writeResourcesToDisk();

    const filePath = isWindows
      ? pathToFileURL(path.join(outputPath, fileName))
      : path.join(outputPath, fileName);

    // Change to vm.module of node or loaders as far as it is stable
    const userConfig = (await import(filePath as string)).default;
    try {
      fse.unlink(filePath, () => void 0);
    } catch {
      /** do nothing */
    }

    const config = await (typeof userConfig === 'function'
      ? userConfig(configEnv)
      : userConfig);

    if (!config.root) {
      config.root = inlineOptions.root;
    }

    if (!isObject(config)) {
      throw new Error(`config must export or return an object.`);
    }
    return config;
  }
}

export function normalizePublicDir(root: string, userPublicDir?: string) {
  const publicDir = userPublicDir ?? 'public';
  const absPublicDirPath = path.isAbsolute(publicDir)
    ? publicDir
    : path.join(root, publicDir);
  return absPublicDirPath;
}

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

export async function resolveMergedUserConfig(
  mergedUserConfig: UserConfig,
  configFilePath: string | undefined,
  mode: 'development' | 'production' | string,
  logger: Logger = new Logger()
): Promise<ResolvedUserConfig> {
  const resolvedUserConfig = {
    ...mergedUserConfig,
    compilation: {
      ...mergedUserConfig.compilation,
      external: []
    }
  } as ResolvedUserConfig;

  // set internal config
  resolvedUserConfig.envMode = mode;

  if (configFilePath) {
    const dependencies = await traceDependencies(configFilePath, logger);
    dependencies.sort();
    resolvedUserConfig.configFileDependencies = dependencies;
    resolvedUserConfig.configFilePath = configFilePath;
  }

  const resolvedRootPath = resolvedUserConfig.root ?? process.cwd();
  const resolvedEnvPath = resolvedUserConfig.envDir
    ? resolvedUserConfig.envDir
    : resolvedRootPath;

  const userEnv = loadEnv(
    resolvedUserConfig.envMode ?? mode,
    resolvedEnvPath,
    resolvedUserConfig.envPrefix
  );
  const existsEnvFiles = getExistsEnvFiles(
    resolvedUserConfig.envMode ?? mode,
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
    NODE_ENV: mergedUserConfig.compilation.mode ?? mode,
    mode: mode
  };

  return resolvedUserConfig;
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
  logger: Logger = new Logger()
): Promise<{ config: any; configFilePath: string } | undefined> {
  const { root = '.' } = inlineOptions;
  const configRootPath = path.resolve(root);
  let resolvedPath: string | undefined;
  try {
    if (configFile) {
      resolvedPath = path.resolve(root, configFile);
    } else {
      resolvedPath = await getConfigFilePath(configRootPath);
    }

    const config = await readConfigFile(
      inlineOptions,
      resolvedPath,
      configEnv,
      logger
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
      logger.error(
        `Failed to load config file: ${errorMessage} \n${stackTrace}`,
        {
          exit: true
        }
      );
    }
    const potentialSolution =
      'Potential solutions: \n1. Try set `FARM_CONFIG_FORMAT=cjs`(default to esm)\n2. Try set `FARM_CONFIG_FULL_BUNDLE=1`';
    throw new Error(
      `Failed to load farm config file: ${errorMessage}. \n ${potentialSolution} \n ${error.stack}`
    );
  }
}

function checkCompilationInputValue(userConfig: UserConfig, logger: Logger) {
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
    if (isTargetNode) {
      // If input is not specified, try to find index.js or index.ts
      const entryFiles = ['./index.js', './index.ts'];

      for (const entryFile of entryFiles) {
        try {
          if (fs.statSync(path.resolve(userConfig?.root, entryFile))) {
            inputIndexConfig = { index: entryFile };
            break;
          }
        } catch (error) {
          errorMessage = error.stack;
        }
      }
    } else {
      try {
        if (fs.statSync(path.resolve(userConfig?.root, defaultHtmlPath))) {
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
        }  from ${userConfig.root}. \n${errorMessage}`,
        { exit: true }
      );
    }
  }

  return inputIndexConfig;
}

export async function getConfigFilePath(
  configRootPath: string
): Promise<string | undefined> {
  if (fse.statSync(configRootPath).isDirectory()) {
    for (const name of DEFAULT_CONFIG_NAMES) {
      const resolvedPath = path.join(configRootPath, name);
      const isFile =
        fse.existsSync(resolvedPath) && fse.statSync(resolvedPath).isFile();

      if (isFile) {
        return resolvedPath;
      }
    }
  }
  return undefined;
}

export async function resolvePlugins(
  userConfig: UserConfig,
  logger: Logger,
  mode: CompilationMode
) {
  const { jsPlugins, rustPlugins } = await resolveFarmPlugins(userConfig);
  const rawJsPlugins = (await resolveAsyncPlugins(jsPlugins || [])).filter(
    Boolean
  );

  let vitePluginAdapters: JsPlugin[] = [];
  const vitePlugins = (userConfig?.vitePlugins ?? []).filter(Boolean);

  if (vitePlugins.length) {
    vitePluginAdapters = await handleVitePlugins(
      vitePlugins,
      userConfig,
      logger,
      mode
    );
  }

  return {
    jsPlugins: rawJsPlugins,
    vitePlugins,
    rustPlugins,
    vitePluginAdapters
  };
}

export async function resolveDefaultUserConfig(options: any) {
  const { inlineOptions, format, outputPath, fileName, configFilePath } =
    options;
  const defaultConfig: UserConfig = {
    root: path.resolve(inlineOptions.root ?? '.'),
    compilation: {
      input: {
        [fileName]: configFilePath
      },
      output: {
        entryFilename: '[entryName]',
        path: outputPath,
        format,
        targetEnv: 'node'
      },
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
      watch: false,
      sourcemap: false,
      treeShaking: false,
      minify: false,
      presetEnv: false,
      lazyCompilation: false,
      persistentCache: false,
      progress: false
    }
  };

  const resolvedUserConfig: ResolvedUserConfig = await resolveUserConfig(
    defaultConfig,
    undefined,
    'development'
  );

  const normalizedConfig = await normalizeUserCompilationConfig(
    resolvedUserConfig,
    'development'
  );

  return normalizedConfig;
}

export async function resolveUserConfig(
  userConfig: UserConfig,
  configFilePath: string | undefined,
  mode: 'development' | 'production' | string,
  logger: Logger = new Logger()
): Promise<ResolvedUserConfig> {
  const resolvedUserConfig = {
    ...userConfig
  } as ResolvedUserConfig;

  // set internal config
  resolvedUserConfig.envMode = mode;

  if (configFilePath) {
    const dependencies = await traceDependencies(configFilePath, logger);
    dependencies.sort();
    resolvedUserConfig.configFileDependencies = dependencies;
    resolvedUserConfig.configFilePath = configFilePath;
  }

  const resolvedRootPath = resolvedUserConfig.root ?? process.cwd();
  const resolvedEnvPath = resolvedUserConfig.envDir
    ? resolvedUserConfig.envDir
    : resolvedRootPath;

  const userEnv = loadEnv(
    resolvedUserConfig.envMode ?? mode,
    resolvedEnvPath,
    resolvedUserConfig.envPrefix
  );
  const existsEnvFiles = getExistsEnvFiles(
    resolvedUserConfig.envMode ?? mode,
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
    NODE_ENV: userConfig.compilation.mode ?? mode,
    mode: mode
  };

  return resolvedUserConfig;
}
