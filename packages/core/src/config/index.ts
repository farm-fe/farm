import crypto from 'node:crypto';
import fs from 'node:fs';
import module from 'node:module';
import path, { isAbsolute, join } from 'node:path';
import { pathToFileURL } from 'node:url';

import { bindingPath } from '../../binding/index.js';
import { type PluginTransformHookParam } from '../types/binding.js';

import { JsPlugin } from '../index.js';
import {
  getSortedPlugins,
  handleVitePlugins,
  resolveAsyncPlugins,
  resolveConfigHook,
  resolveConfigResolvedHook,
  resolveFarmPlugins
} from '../plugin/index.js';
import { Server } from '../server/index.js';
import { urlRegex } from '../utils/http.js';
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
import { CompilationMode, loadEnv, setProcessEnv } from './env.js';
import { normalizeOutput } from './normalize-config/normalize-output.js';
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
  FarmCLIOptions,
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

async function getDefaultConfig(
  config: UserConfig,
  inlineOptions: FarmCLIOptions,
  logger: Logger,
  mode?: CompilationMode
) {
  const resolvedUserConfig = await resolveMergedUserConfig(
    config,
    undefined,
    inlineOptions.mode ?? mode,
    logger
  );

  resolvedUserConfig.server = normalizeDevServerOptions({}, mode);

  resolvedUserConfig.compilation = await normalizeUserCompilationConfig(
    resolvedUserConfig,
    config,
    logger,
    mode,
    true
  );
  resolvedUserConfig.root = resolvedUserConfig.compilation.root;
  resolvedUserConfig.jsPlugins = [];
  resolvedUserConfig.rustPlugins = [];

  return resolvedUserConfig;
}

async function handleServerPortConflict(
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
  inlineOptions: FarmCLIOptions & UserConfig,
  logger: Logger,
  mode?: CompilationMode,
  isHandleServerPortConflict = true
): Promise<ResolvedUserConfig> {
  // Clear the console according to the cli command
  checkClearScreen(inlineOptions);
  inlineOptions.mode = inlineOptions.mode ?? mode;

  // configPath may be file or directory
  let { configPath } = inlineOptions;
  let rawConfig: UserConfig = mergeFarmCliConfig(inlineOptions, {});

  // if the config file can not found, just merge cli options and return default
  if (configPath) {
    if (!path.isAbsolute(configPath)) {
      throw new Error('configPath must be an absolute path');
    }

    const loadedUserConfig = await loadConfigFile(
      configPath,
      inlineOptions,
      logger,
      mode
    );
    if (loadedUserConfig) {
      configPath = loadedUserConfig.configFilePath;
      rawConfig = mergeConfig(rawConfig, loadedUserConfig.config);
    }
    rawConfig.compilation.mode =
      loadedUserConfig?.config?.compilation?.mode ?? mode;
  } else {
    mergeConfig(
      rawConfig,
      await getDefaultConfig(rawConfig, inlineOptions, logger, mode)
    );
  }

  const { config: userConfig, configFilePath } = {
    configFilePath: configPath,
    config: rawConfig
  };

  const { jsPlugins, rustPlugins } = await resolveFarmPlugins(userConfig);

  const rawJsPlugins = (await resolveAsyncPlugins(jsPlugins || [])).filter(
    Boolean
  );

  let vitePluginAdapters: JsPlugin[] = [];
  const vitePlugins = (userConfig?.vitePlugins ?? []).filter(Boolean);
  // run config and configResolved hook
  if (vitePlugins.length) {
    vitePluginAdapters = await handleVitePlugins(
      vitePlugins,
      userConfig,
      logger,
      mode
    );
  }

  const sortFarmJsPlugins = getSortedPlugins([
    ...rawJsPlugins,
    ...vitePluginAdapters,
    externalAdapter()
  ]);

  const config = await resolveConfigHook(userConfig, sortFarmJsPlugins);

  const mergedUserConfig = mergeFarmCliConfig(inlineOptions, config);

  const resolvedUserConfig = await resolveMergedUserConfig(
    mergedUserConfig,
    configFilePath,
    inlineOptions.mode ?? mode,
    logger
  );

  // normalize server config first cause it may be used in normalizeUserCompilationConfig
  resolvedUserConfig.server = normalizeDevServerOptions(
    resolvedUserConfig.server,
    mode
  );

  if (isHandleServerPortConflict) {
    await handleServerPortConflict(resolvedUserConfig, logger, mode);
  }

  resolvedUserConfig.compilation = await normalizeUserCompilationConfig(
    resolvedUserConfig,
    mergedUserConfig,
    logger,
    mode
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

  // TODO Temporarily solve the problem of alias adaptation to vite
  if (resolvedUserConfig.compilation?.resolve?.alias && vitePlugins.length) {
    resolvedUserConfig.compilation.resolve.alias = transformAliasWithVite(
      resolvedUserConfig.compilation.resolve.alias as unknown as Array<Alias>
    );
  }

  return resolvedUserConfig;
}

// type ServerConfig = {
//   server?: NormalizedServerConfig;
// };

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
  userConfig: UserConfig,
  logger: Logger,
  mode: CompilationMode = 'development',
  isDefault = false
): Promise<ResolvedCompilation> {
  const { compilation, root } = resolvedUserConfig;

  // resolve root path
  const resolvedRootPath = normalizePath(root);

  resolvedUserConfig.root = resolvedRootPath;

  // resolve public path
  if (compilation?.output?.publicPath) {
    compilation.output.publicPath = normalizePublicPath(
      compilation.output.publicPath,
      logger
    );
  }

  if (!userConfig.compilation) {
    userConfig.compilation = {};
  }

  // if normalize default config, skip check input option
  const inputIndexConfig = !isDefault
    ? checkCompilationInputValue(userConfig, logger)
    : {};

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

  normalizeOutput(resolvedCompilation, isProduction);
  normalizeExternal(userConfig, resolvedCompilation);

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
    const packageJsonExists = fs.existsSync(packageJsonPath);
    const namespaceName = packageJsonExists
      ? JSON.parse(fs.readFileSync(packageJsonPath, { encoding: 'utf-8' }))
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
    Object.values(resolvedCompilation.input).some((value) =>
      value.endsWith('.html')
    );
  if (
    resolvedCompilation.output.targetEnv !== 'node' &&
    isArray(resolvedCompilation.runtime.plugins) &&
    resolvedUserConfig.server?.hmr &&
    is_entry_html &&
    !resolvedCompilation.runtime.plugins.includes(hmrClientPluginPath)
  ) {
    const publicPath =
      resolvedUserConfig.compilation?.output?.publicPath ?? '/';
    const hmrPath = resolvedUserConfig.server.hmr.path;
    const serverOptions = resolvedUserConfig.server;
    const defineHmrPath = normalizeBasePath(path.join(publicPath, hmrPath));

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
        'swc_core v0.90'
      )}, please make sure the plugin is ${colors.green(
        'compatible'
      )} with swc_core ${colors.yellow(
        'swc_core v0.90'
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

  // normalize persistent cache at last
  await normalizePersistentCache(
    resolvedCompilation,
    resolvedUserConfig,
    logger
  );

  return resolvedCompilation;
}

export const DEFAULT_HMR_OPTIONS: Required<UserHmrConfig> = {
  host: true,
  port:
    (process.env.FARM_DEFAULT_HMR_PORT &&
      Number(process.env.FARM_DEFAULT_HMR_PORT)) ??
    undefined,
  path: '/__hmr',
  protocol: 'ws',
  watchOptions: {}
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
  open: false,
  strictPort: false,
  cors: false,
  spa: true,
  middlewares: [],
  writeToDisk: false
};

export const DEFAULT_COMPILATION_OPTIONS: Partial<ResolvedCompilation> = {
  output: {
    path: './dist',
    publicPath: '/'
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
      'css'
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

export function normalizeDevServerOptions(
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

async function readConfigFile(
  inlineOptions: FarmCLIOptions,
  configFilePath: string,
  logger: Logger,
  mode: CompilationMode
): Promise<UserConfig | undefined> {
  if (fs.existsSync(configFilePath)) {
    !__FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ &&
      logger.info(`Using config file at ${bold(green(configFilePath))}`);
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
      .join('')}.mjs`;

    const tsDefaultUserConfig: UserConfig = {
      root: inlineOptions.root,
      compilation: {
        input: {
          [fileName]: configFilePath
        },
        output: {
          entryFilename: '[entryName]',
          path: outputPath,
          format: 'esm',
          targetEnv: 'node'
        },
        external: ['!^(\\./|\\.\\./|[A-Za-z]:\\\\|/).*'],
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
    const tsDefaultResolvedUserConfig: ResolvedUserConfig =
      await resolveMergedUserConfig(
        tsDefaultUserConfig,
        undefined,
        'development',
        logger
      );

    const normalizedConfig = await normalizeUserCompilationConfig(
      tsDefaultResolvedUserConfig,
      tsDefaultUserConfig,
      logger,
      mode as CompilationMode
    );

    const compiler = new Compiler(
      {
        config: normalizedConfig,
        jsPlugins: [replaceDirnamePlugin()],
        rustPlugins: []
      },
      logger
    );

    await compiler.compile();

    compiler.writeResourcesToDisk();

    const filePath = isWindows
      ? pathToFileURL(path.join(outputPath, fileName))
      : path.join(outputPath, fileName);

    // Change to vm.module of node or loaders as far as it is stable
    const userConfig = (await import(filePath as string)).default;
    try {
      fs.unlink(filePath, () => void 0);
    } catch {
      /** do nothing */
    }

    const configEnv = { mode: inlineOptions.mode ?? process.env.NODE_ENV };
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

/**
 * @param publicPath  publicPath option
 * @param logger  logger instance
 * @param isPrefixNeeded  whether to add a prefix to the publicPath
 * @returns  normalized publicPath
 */
export function normalizePublicPath(
  publicPath = '/',
  logger: Logger,
  isPrefixNeeded = true
) {
  let normalizedPublicPath = publicPath;
  let warning = false;
  // normalize relative path
  if (
    normalizedPublicPath.startsWith('.') ||
    normalizedPublicPath.startsWith('..')
  ) {
    warning = true;
    normalizedPublicPath = normalizedPublicPath.replace(/^\.+/, '');
  }

  // normalize appended relative path
  if (!normalizedPublicPath.endsWith('/')) {
    if (!urlRegex.test(normalizedPublicPath)) {
      warning = true;
    }
    normalizedPublicPath = normalizedPublicPath + '/';
  }

  // normalize prepended relative path
  if (
    normalizedPublicPath.startsWith('/') &&
    !urlRegex.test(normalizedPublicPath) &&
    !isPrefixNeeded
  ) {
    normalizedPublicPath = normalizedPublicPath.slice(1);
  } else if (
    isPrefixNeeded &&
    !normalizedPublicPath.startsWith('/') &&
    !urlRegex.test(normalizedPublicPath)
  ) {
    warning = true;
    normalizedPublicPath = '/' + normalizedPublicPath;
  }

  warning &&
    isPrefixNeeded &&
    logger.warn(
      ` (!) Irregular 'publicPath' options: '${publicPath}', it should only be an absolute path like '/publicPath/', an url or an empty string.`
    );

  return normalizedPublicPath;
}

function checkClearScreen(inlineConfig: FarmCLIOptions) {
  if (
    inlineConfig.clearScreen &&
    !__FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__
  ) {
    clearScreen();
  }
}

export async function resolveMergedUserConfig(
  mergedUserConfig: UserConfig,
  configFilePath: string | undefined,
  mode: 'development' | 'production' | string,
  logger: Logger
): Promise<ResolvedUserConfig> {
  const serverConfig: NormalizedServerConfig = {
    ...DEFAULT_DEV_SERVER_OPTIONS,
    ...mergedUserConfig.server,
    hmr: {
      ...DEFAULT_HMR_OPTIONS,
      ...(isObject(mergedUserConfig.server?.hmr)
        ? mergedUserConfig.server.hmr
        : {})
    }
  };
  const resolvedUserConfig: ResolvedUserConfig = {
    ...mergedUserConfig,
    compilation: {
      ...mergedUserConfig.compilation,
      external: []
    },
    server: serverConfig
  };

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

  const [userEnv, existsEnvFiles] = loadEnv(
    resolvedUserConfig.envMode ?? mode,
    resolvedEnvPath,
    resolvedUserConfig.envPrefix
  );

  resolvedUserConfig.envFiles = [
    ...(Array.isArray(resolvedUserConfig.envFiles)
      ? resolvedUserConfig.envFiles
      : []),
    ...existsEnvFiles
  ];

  resolvedUserConfig.env = {
    ...userEnv,
    NODE_ENV: mode
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
  configPath: string,
  inlineOptions: FarmCLIOptions,
  logger: Logger = new Logger(),
  mode: CompilationMode
): Promise<{ config: UserConfig; configFilePath: string } | undefined> {
  // if configPath points to a directory, try to find a config file in it using default config
  try {
    const configFilePath = await getConfigFilePath(configPath);

    if (configFilePath) {
      const config = await readConfigFile(
        inlineOptions,
        configFilePath,
        logger,
        mode
      );

      return {
        config: config && parseUserConfig(config),
        configFilePath: configFilePath
      };
    }
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

    throw new Error(
      `Failed to load farm config file: ${errorMessage} \n ${error.stack}`
    );
  }
}

function checkCompilationInputValue(userConfig: UserConfig, logger: Logger) {
  const { compilation } = userConfig;
  const targetEnv = compilation?.output?.targetEnv;
  const isTargetNode = targetEnv === 'node';
  const defaultHtmlPath = './index.html';
  let inputIndexConfig: { index?: string } = { index: '' };
  let errorMessage = '';

  // Check if input is specified
  if (!isEmptyObject(compilation?.input)) {
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
  configPath: string
): Promise<string | undefined> {
  if (fs.statSync(configPath).isDirectory()) {
    for (const name of DEFAULT_CONFIG_NAMES) {
      const resolvedPath = path.join(configPath, name);
      const isFile =
        fs.existsSync(resolvedPath) && fs.statSync(resolvedPath).isFile();

      if (isFile) {
        return resolvedPath;
      }
    }
  } else if (fs.statSync(configPath).isFile()) {
    return configPath;
  }

  return undefined;
}

// transform __dirname and __filename with resolve config file path
export function replaceDirnamePlugin() {
  const moduleTypes = ['ts', 'js', 'cjs', 'mjs', 'mts', 'cts'];
  const resolvedPaths: string[] = [];
  return {
    name: 'replace-dirname',
    transform: {
      filters: {
        moduleTypes,
        resolvedPaths
      },
      async executor(param: PluginTransformHookParam) {
        const { content, resolvedPath, moduleType } = param;
        let replaceContent = content;
        const dirPath = path.dirname(resolvedPath);

        replaceContent = param.content
          .replace(/__dirname/g, JSON.stringify(dirPath))
          .replace(/__filename/g, JSON.stringify(resolvedPath))
          .replace(
            /import\.meta\.url/g,
            JSON.stringify(pathToFileURL(resolvedPath))
          );

        return {
          content: replaceContent,
          moduleType
        };
      }
    }
  };
}
