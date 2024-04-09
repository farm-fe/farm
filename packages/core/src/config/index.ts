import module from 'node:module';
import fs from 'node:fs';
import path, { isAbsolute, join } from 'node:path';
import crypto from 'node:crypto';
import { pathToFileURL } from 'node:url';

import {
  getSortedPlugins,
  handleVitePlugins,
  resolveAsyncPlugins,
  resolveConfigHook,
  resolveConfigResolvedHook,
  resolveFarmPlugins
} from '../plugin/index.js';
import { bindingPath, Config } from '../../binding/index.js';
import { Server } from '../server/index.js';
import { parseUserConfig } from './schema.js';
import { CompilationMode, loadEnv, setProcessEnv } from './env.js';
import { __FARM_GLOBAL__ } from './_global.js';
import {
  bold,
  clearScreen,
  Logger,
  green,
  isArray,
  isEmptyObject,
  isObject,
  isWindows,
  normalizePath,
  normalizeBasePath,
  getAliasEntries,
  transformAliasWithVite
} from '../utils/index.js';
import { urlRegex } from '../utils/http.js';
import { JsPlugin } from '../index.js';
import { normalizePersistentCache } from './normalize-config/normalize-persistent-cache.js';
import { normalizeOutput } from './normalize-config/normalize-output.js';
import { traceDependencies } from '../utils/trace-dependencies.js';

import type {
  Alias,
  FarmCLIOptions,
  FarmCLIServerOptions,
  NormalizedServerConfig,
  ResolvedUserConfig,
  UserConfig,
  UserConfigExport,
  UserHmrConfig,
  UserServerConfig
} from './types.js';
import { normalizeExternal } from './normalize-config/normalize-external.js';
import { DEFAULT_CONFIG_NAMES, FARM_DEFAULT_NAMESPACE } from './constants.js';
import merge from '../utils/merge.js';

export * from './types.js';
export function defineFarmConfig(config: UserConfig): UserConfig;
export function defineFarmConfig(
  config: Promise<UserConfig>
): Promise<UserConfig>;
export function defineFarmConfig(config: UserConfigExport): UserConfigExport;
export function defineFarmConfig(config: UserConfigExport): UserConfigExport {
  return config;
}

async function getDefaultConfig(
  inlineOptions: FarmCLIOptions,
  logger: Logger,
  mode?: CompilationMode,
  isHandleServerPortConflict = true
) {
  const mergedUserConfig = mergeInlineCliOptions({}, inlineOptions);

  const resolvedUserConfig = await resolveMergedUserConfig(
    mergedUserConfig,
    undefined,
    inlineOptions.mode ?? mode
  );
  resolvedUserConfig.server = normalizeDevServerOptions({}, mode);

  if (isHandleServerPortConflict) {
    await handleServerPortConflict(resolvedUserConfig, logger, mode);
  }

  resolvedUserConfig.compilation = await normalizeUserCompilationConfig(
    resolvedUserConfig,
    logger,
    mode
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
  inlineOptions: FarmCLIOptions,
  logger: Logger,
  mode?: CompilationMode,
  isHandleServerPortConflict = true
): Promise<ResolvedUserConfig> {
  // Clear the console according to the cli command
  checkClearScreen(inlineOptions);
  inlineOptions.mode = inlineOptions.mode ?? mode;

  // configPath may be file or directory
  const { configPath } = inlineOptions;
  // if the config file can not found, just merge cli options and return default
  if (!configPath) {
    return getDefaultConfig(
      inlineOptions,
      logger,
      mode,
      isHandleServerPortConflict
    );
  }

  if (!path.isAbsolute(configPath)) {
    throw new Error('configPath must be an absolute path');
  }

  const loadedUserConfig = await loadConfigFile(
    configPath,
    inlineOptions,
    logger
  );

  if (!loadedUserConfig) {
    return getDefaultConfig(
      inlineOptions,
      logger,
      mode,
      isHandleServerPortConflict
    );
  }

  const { config: userConfig, configFilePath } = loadedUserConfig;

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
    ...vitePluginAdapters
  ]);

  const config = await resolveConfigHook(userConfig, sortFarmJsPlugins);

  const mergedUserConfig = mergeInlineCliOptions(config, inlineOptions);

  const resolvedUserConfig = await resolveMergedUserConfig(
    mergedUserConfig,
    configFilePath,
    inlineOptions.mode ?? mode
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
    logger,
    mode
  );
  resolvedUserConfig.root = resolvedUserConfig.compilation.root;
  resolvedUserConfig.jsPlugins = sortFarmJsPlugins;
  resolvedUserConfig.rustPlugins = rustPlugins;

  // Temporarily dealing with alias objects and arrays in js will be unified in rust in the future.]
  if (resolvedUserConfig.compilation.resolve?.alias && vitePlugins.length) {
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

type ServerConfig = {
  server?: NormalizedServerConfig;
};

/**
 * Normalize user config and transform it to rust compiler compatible config
 * @param config
 * @returns resolved config that parsed to rust compiler
 */
export async function normalizeUserCompilationConfig(
  userConfig: ResolvedUserConfig,
  logger: Logger,
  mode: CompilationMode = 'development'
): Promise<Config['config']> {
  const { compilation, root } = userConfig;
  // resolve root path
  const resolvedRootPath = normalizePath(
    root ? path.resolve(root) : process.cwd()
  );

  // resolve public path
  if (compilation?.output?.publicPath) {
    compilation.output.publicPath = normalizePublicPath(
      compilation.output.publicPath,
      logger
    );
  }

  const inputIndexConfig = checkCompilationInputValue(userConfig, logger);
  const config: Config['config'] & ServerConfig = merge(
    {},
    DEFAULT_COMPILATION_OPTIONS,
    {
      input: inputIndexConfig
    },
    compilation
  );

  if (!config.root) {
    config.root = resolvedRootPath;
  }

  const isProduction = mode === 'production';
  const isDevelopment = mode === 'development';
  config.mode = config.mode ?? mode;

  config.coreLibPath = bindingPath;

  normalizeOutput(config, isProduction);
  normalizeExternal(config);

  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore do not check type for this internal option
  if (!config.assets?.publicDir) {
    if (!config.assets) {
      config.assets = {};
    }

    const userPublicDir = userConfig.publicDir
      ? userConfig.publicDir
      : join(config.root, 'public');

    if (isAbsolute(userPublicDir)) {
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore do not check type for this internal option
      config.assets.publicDir = userPublicDir;
    } else {
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore do not check type for this internal option
      config.assets.publicDir = join(config.root, userPublicDir);
    }
  }

  config.define = Object.assign(
    {
      // skip self define
      ['FARM' + '_PROCESS_ENV']: userConfig.env
    },
    config?.define,
    // for node target, we should not define process.env.NODE_ENV
    config.output?.targetEnv === 'node'
      ? {}
      : Object.keys(userConfig.env || {}).reduce((env: any, key) => {
          env[`$__farm_regex:(global(This)?\\.)?process\\.env\\.${key}`] =
            userConfig.env[key];
          return env;
        }, {})
  );

  const require = module.createRequire(import.meta.url);
  const hmrClientPluginPath = require.resolve('@farmfe/runtime-plugin-hmr');
  const ImportMetaPluginPath = require.resolve(
    '@farmfe/runtime-plugin-import-meta'
  );

  if (!config.runtime) {
    config.runtime = {
      path: require.resolve('@farmfe/runtime'),
      plugins: []
    };
  }

  if (!config.runtime.path) {
    config.runtime.path = require.resolve('@farmfe/runtime');
  }

  if (!config.runtime.swcHelpersPath) {
    config.runtime.swcHelpersPath = path.dirname(
      require.resolve('@swc/helpers/package.json')
    );
  }

  if (!config.runtime.plugins) {
    config.runtime.plugins = [];
  } else {
    // make sure all plugin paths are absolute
    config.runtime.plugins = config.runtime.plugins.map((plugin) => {
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
  if (!config.runtime.namespace) {
    // read package.json name field
    const packageJsonPath = path.resolve(resolvedRootPath, 'package.json');
    const packageJsonExists = fs.existsSync(packageJsonPath);
    const namespaceName = packageJsonExists
      ? JSON.parse(fs.readFileSync(packageJsonPath, { encoding: 'utf-8' }))
          ?.name ?? FARM_DEFAULT_NAMESPACE
      : FARM_DEFAULT_NAMESPACE;

    config.runtime.namespace = crypto
      .createHash('md5')
      .update(namespaceName)
      .digest('hex');
  }

  if (isProduction) {
    config.lazyCompilation = false;
  } else if (config.lazyCompilation === undefined) {
    if (isDevelopment) {
      config.lazyCompilation = true;
    } else {
      config.lazyCompilation = false;
    }
  }

  if (config.mode === undefined) {
    config.mode = mode;
  }

  setProcessEnv(config.mode);

  if (
    config.output.targetEnv !== 'node' &&
    isArray(config.runtime.plugins) &&
    userConfig.server.hmr &&
    !config.runtime.plugins.includes(hmrClientPluginPath)
  ) {
    const publicPath = userConfig.compilation?.output?.publicPath ?? '/';
    const hmrPath = userConfig.server.hmr.path;
    const serverOptions = userConfig.server;
    const defineHmrPath = normalizeBasePath(path.join(publicPath, hmrPath));

    config.runtime.plugins.push(hmrClientPluginPath);
    // TODO optimize get hmr logic
    config.define.FARM_HMR_PORT = String(
      (serverOptions.hmr.port || undefined) ??
        serverOptions.port ??
        DEFAULT_DEV_SERVER_OPTIONS.port
    );
    config.define.FARM_HMR_HOST = userConfig.server.hmr.host;
    config.define.FARM_HMR_PROTOCOL = userConfig.server.hmr.protocol;
    config.define.FARM_HMR_PATH = defineHmrPath;
  }

  if (
    isArray(config.runtime.plugins) &&
    !config.runtime.plugins.includes(ImportMetaPluginPath)
  ) {
    config.runtime.plugins.push(ImportMetaPluginPath);
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

    config.input = input;
  }

  if (config.treeShaking === undefined) {
    if (isProduction) {
      config.treeShaking = true;
    } else {
      config.treeShaking = false;
    }
  }

  // lazyCompilation should be disabled in production mode
  // so, it only happens in development mode
  // https://github.com/farm-fe/farm/issues/962
  if (config.treeShaking && config.lazyCompilation) {
    logger.error(
      'treeShaking option is not supported in lazyCompilation mode, treeShaking will be disabled.'
    );
    config.treeShaking = false;
  }

  if (config.minify === undefined) {
    if (isProduction) {
      config.minify = true;
    } else {
      config.minify = false;
    }
  }

  if (config.presetEnv === undefined) {
    if (isProduction) {
      config.presetEnv = true;
    } else {
      config.presetEnv = false;
    }
  }

  // normalize persistent cache at last
  await normalizePersistentCache(config, userConfig);

  return config;
}

export const DEFAULT_HMR_OPTIONS: Required<UserHmrConfig> = {
  ignores: [],
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

export const DEFAULT_COMPILATION_OPTIONS: Partial<Config['config']> = {
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
  logger: Logger
): Promise<UserConfig | undefined> {
  if (fs.existsSync(configFilePath)) {
    let userConfig: UserConfigExport;
    !__FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ &&
      logger.info(`Using config file at ${bold(green(configFilePath))}`);
    // if config is written in typescript, we need to compile it to javascript using farm first
    if (configFilePath.endsWith('.ts')) {
      const Compiler = (await import('../compiler/index.js')).Compiler;
      const outputPath = path.join(
        path.dirname(configFilePath),
        'node_modules',
        '.farm'
      );

      const fileName = `farm.config.bundle-{${Date.now()}-${Math.random()
        .toString(16)
        .split('.')
        .join('')}}.mjs`;

      const normalizedConfig = await normalizeUserCompilationConfig(
        {
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
        },
        logger,
        inlineOptions.mode as CompilationMode
      );

      const compiler = new Compiler({
        config: normalizedConfig,
        jsPlugins: [],
        rustPlugins: []
      });

      await compiler.compile();

      compiler.writeResourcesToDisk();

      const filePath = isWindows
        ? pathToFileURL(path.join(outputPath, fileName))
        : path.join(outputPath, fileName);

      try {
        // Change to vm.module of node or loaders as far as it is stable
        userConfig = (await import(filePath as string)).default;
      } finally {
        // fs.unlink(filePath, () => void 0);
      }
    } else {
      const filePath = isWindows
        ? pathToFileURL(configFilePath)
        : configFilePath;
      // Change to vm.module of node or loaders as far as it is stable
      userConfig = (await import(filePath as string)).default;
    }
    const configEnv = { mode: inlineOptions.mode ?? process.env.NODE_ENV };
    const config = await (typeof userConfig === 'function'
      ? userConfig(configEnv)
      : userConfig);
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

function mergeInlineCliOptions(
  userConfig: UserConfig,
  inlineOptions: FarmCLIOptions
): UserConfig {
  if (inlineOptions.root) {
    const cliRoot = inlineOptions.root;

    if (!isAbsolute(cliRoot)) {
      userConfig.root = path.resolve(process.cwd(), cliRoot);
    } else {
      userConfig.root = cliRoot;
    }
  }

  if (userConfig.root && !isAbsolute(userConfig.root)) {
    const resolvedRoot = path.resolve(
      inlineOptions.configPath || process.cwd(),
      userConfig.root
    );
    userConfig.root = resolvedRoot;
  }

  // set compiler options
  ['minify', 'sourcemap'].forEach((option: keyof FarmCLIOptions) => {
    if (inlineOptions[option] !== undefined) {
      userConfig.compilation = {
        ...(userConfig.compilation ?? {}),
        [option]: inlineOptions[option]
      };
    }
  });

  const outputOptions = inlineOptions.compilation?.output;

  if (outputOptions?.path) {
    userConfig.compilation = {
      ...(userConfig.compilation ?? {})
    };

    userConfig.compilation.output = {
      ...(userConfig.compilation.output ?? {}),
      path: outputOptions?.path
    };
  }

  if (outputOptions?.targetEnv) {
    userConfig.compilation = {
      ...(userConfig.compilation ?? {})
    };

    userConfig.compilation.output = {
      ...(userConfig.compilation.output ?? {}),
      targetEnv: outputOptions?.targetEnv
    };
  }

  // set server options
  ['port', 'open', 'https', 'hmr', 'host', 'strictPort'].forEach(
    (option: keyof FarmCLIServerOptions) => {
      if (inlineOptions.server?.[option]) {
        userConfig.server = {
          ...(userConfig.server ?? {}),
          [option]: inlineOptions.server[option]
        };
      }
    }
  );

  return userConfig;
}

async function resolveMergedUserConfig(
  mergedUserConfig: UserConfig,
  configFilePath: string | undefined,
  mode: 'development' | 'production' | string
) {
  const resolvedUserConfig = { ...mergedUserConfig } as ResolvedUserConfig;

  // set internal config
  resolvedUserConfig.envMode = mode;

  if (configFilePath) {
    const dependencies = await traceDependencies(configFilePath);
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
  logger: Logger = new Logger()
): Promise<{ config: UserConfig; configFilePath: string } | undefined> {
  // if configPath points to a directory, try to find a config file in it using default config
  try {
    const configFilePath = await getConfigFilePath(configPath);

    if (configFilePath) {
      const config = await readConfigFile(
        inlineOptions,
        configFilePath,
        logger
      );
      return {
        config: config && parseUserConfig(config),
        configFilePath: configFilePath
      };
    }
  } catch (error) {
    // In this place, the original use of
    // throw caused emit to the outermost catch
    // callback, causing the code not to execute.
    // If the internal catch compiler's own
    // throw error can solve this problem,
    // it will not continue to affect the execution of
    // external code. We just need to return the default config.
    if (inlineOptions.mode === 'production') {
      logger.error(`Failed to load config file: \n ${error.stack}`, {
        exit: true
      });
    }

    throw new Error(
      'Failed to load farm config file: ' + error + ' ' + error.stack
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
          if (
            fs.statSync(
              path.resolve(userConfig?.root ?? process.cwd(), entryFile)
            )
          ) {
            inputIndexConfig = { index: entryFile };
            break;
          }
        } catch (error) {
          errorMessage = error.stack;
        }
      }
    } else {
      try {
        if (
          fs.statSync(
            path.resolve(userConfig?.root ?? process.cwd(), defaultHtmlPath)
          )
        ) {
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
