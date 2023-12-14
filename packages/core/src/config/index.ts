import module from 'node:module';
import fs from 'node:fs';
import path, { isAbsolute, join } from 'node:path';
import crypto from 'node:crypto';
import { pathToFileURL } from 'node:url';

import merge from 'lodash.merge';

import {
  convertPlugin,
  getSortedPlugins,
  handleVitePlugins,
  resolveAsyncPlugins,
  resolveConfigHook,
  resolveConfigResolvedHook,
  rustPluginResolver
} from '../plugin/index.js';
import { bindingPath, Config } from '../../binding/index.js';
import { DevServer } from '../server/index.js';
import { parseUserConfig } from './schema.js';
import { CompilationMode, loadEnv, setProcessEnv } from './env.js';
import { __FARM_GLOBAL__ } from './_global.js';
import {
  bold,
  clearScreen,
  green,
  isArray,
  isEmptyObject,
  isObject,
  isWindows,
  Logger,
  normalizePath
} from '../utils/index.js';
import { normalizePersistentCache } from './normalize-config/normalize-persistent-cache.js';
import { normalizeOutput } from './normalize-config/normalize-output.js';
import { traceDependencies } from '../utils/trace-dependencies.js';

import type {
  ConfigEnv,
  FarmCLIOptions,
  NormalizedServerConfig,
  ResolveConfigType,
  ResolvedUserConfig,
  UserConfig,
  UserHmrConfig,
  UserServerConfig
} from './types.js';
import { urlRegex } from '../utils/http.js';
import { JsPlugin } from '../index.js';

export * from './types.js';
export const DEFAULT_CONFIG_NAMES = [
  'farm.config.ts',
  'farm.config.js',
  'farm.config.mjs'
];

export function defineFarmConfig(
  config: UserConfig | Promise<UserConfig>
): UserConfig | Promise<UserConfig> {
  return config;
}

/**
 * Resolve and load user config from the specified path
 * @param configPath
 */
export async function resolveConfig(
  inlineOptions: FarmCLIOptions,
  logger: Logger,
  command: 'serve' | 'build',
  mode?: CompilationMode
): Promise<ResolveConfigType> {
  // Clear the console according to the cli command
  checkClearScreen(inlineOptions);

  let userConfig: ResolvedUserConfig = {};
  // configPath may be file or directory
  const { configPath, root } = inlineOptions;

  if (!configPath) {
    return mergeUserConfig(userConfig, inlineOptions);
  }

  if (!path.isAbsolute(configPath)) {
    throw new Error('configPath must be an absolute path');
  }

  userConfig = await loadFileConfig(userConfig, inlineOptions, logger);

  if (!userConfig.root) {
    userConfig.root = root || process.cwd();
  }
  userConfig.mode = userConfig.compilation?.mode || mode;
  userConfig.isBuild = command === 'build';
  userConfig.command = command;
  const configEnv: ConfigEnv = {
    mode,
    command
  };

  const { jsPlugins, rustPlugins } = await resolveFarmPlugins(userConfig);

  const rawJsPlugins = (await resolveAsyncPlugins(jsPlugins || [])).filter(
    Boolean
  );

  let vitePluginAdapters: JsPlugin[] = [];
  const vitePlugins = userConfig?.vitePlugins ?? [];
  // run config and configResolved hook
  if (vitePlugins.length) {
    vitePluginAdapters = await handleVitePlugins(vitePlugins, userConfig);
  }

  const sortFarmJsPlugins = getSortedPlugins([
    ...rawJsPlugins,
    ...vitePluginAdapters
  ]);

  // TODO vite plugin hook need sort by `order` in config hooks !!! not priority or enforce
  // Start running config hook for all plugins
  const config = await resolveConfigHook(
    userConfig,
    configEnv,
    sortFarmJsPlugins
  );

  // check port availability: auto increment the port if a conflict occurs
  const targetWeb = !(
    userConfig.compilation?.output?.targetEnv === 'node' || userConfig.isBuild
  );
  targetWeb && (await DevServer.resolvePortConflict(userConfig, logger));

  const normalizedConfig = await normalizeUserCompilationConfig(
    inlineOptions,
    config,
    logger,
    mode
  );

  await resolveConfigResolvedHook(normalizedConfig, sortFarmJsPlugins); // Fix: Await the Promise<void> and pass the resolved value to the function.

  return {
    config,
    normalizedConfig: {
      ...normalizedConfig,
      jsPlugins: sortFarmJsPlugins,
      rustPlugins
    }
  };
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
  inlineConfig: (FarmCLIOptions & UserConfig) | null,
  userConfig: ResolvedUserConfig,
  logger: Logger,
  mode: CompilationMode = 'development'
): Promise<Config> {
  const { compilation, root, server, envDir, envPrefix } = userConfig;

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
    {
      input: inputIndexConfig,
      output: {
        path: './dist',
        publicPath: '/'
      },
      sourcemap: true
    },
    compilation
  );

  if (!config.root) {
    config.root = resolvedRootPath;
  }

  config.mode = config.mode ?? mode;
  const isProduction = config.mode === 'production';
  const isDevelopment = config.mode === 'development';

  config.coreLibPath = bindingPath;
  config.configFilePath = userConfig.configFilePath;

  const resolvedEnvPath = envDir ? envDir : resolvedRootPath;

  const [userEnv, existsEnvFiles] = loadEnv(
    inlineConfig?.mode ?? mode,
    resolvedEnvPath,
    envPrefix
  );

  config.envFiles = [
    ...(Array.isArray(config.envFiles) ? config.envFiles : []),
    ...existsEnvFiles
  ];

  config.external = [
    ...module.builtinModules.map((m) => `^${m}$`),
    ...module.builtinModules.map((m) => `^node:${m}$`),
    ...(Array.isArray(config.external) ? config.external : [])
  ];

  normalizeOutput(config, isProduction);

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

  config.env = {
    ...userEnv,
    NODE_ENV: process.env.NODE_ENV || mode
  };

  config.define = Object.assign(
    {
      // skip self define
      ['FARM' + '_PROCESS_ENV']: config.env
    },
    userConfig.define,
    config?.define,
    // for node target, we should not define process.env.NODE_ENV
    config.output?.targetEnv === 'node'
      ? {}
      : Object.keys(config.env).reduce((env: any, key) => {
          env[`process.env.${key}`] = config.env[key];
          return env;
        }, {})
  );

  await normalizePersistentCache(config, userConfig);

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
  }
  // set namespace to package.json name field's hash
  if (!config.runtime.namespace) {
    // read package.json name field
    const packageJsonPath = path.resolve(resolvedRootPath, 'package.json');
    if (fs.existsSync(packageJsonPath)) {
      const packageJson = JSON.parse(
        fs.readFileSync(packageJsonPath, { encoding: 'utf-8' })
      );
      config.runtime.namespace = crypto
        .createHash('md5')
        .update(packageJson.name)
        .digest('hex');
    }
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

  // TODO resolve other server port
  const normalizedDevServerConfig = normalizeDevServerOptions(server, mode);
  config.server = normalizedDevServerConfig;
  if (
    config.output.targetEnv !== 'node' &&
    isArray(config.runtime.plugins) &&
    normalizedDevServerConfig.hmr &&
    !config.runtime.plugins.includes(hmrClientPluginPath)
  ) {
    config.runtime.plugins.push(hmrClientPluginPath);
    config.define.FARM_HMR_PORT = String(normalizedDevServerConfig.hmr.port);
    config.define.FARM_HMR_HOST = normalizedDevServerConfig.hmr.host;
    config.define.FARM_HMR_PATH = normalizedDevServerConfig.hmr.path;
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

  if (config.minify === undefined) {
    if (isProduction) {
      config.minify = true;
    } else {
      config.minify = false;
    }
  }

  if (config.presetEnv === undefined && config.output?.targetEnv !== 'node') {
    if (isProduction) {
      config.presetEnv = true;
    } else {
      config.presetEnv = false;
    }
  }

  return { config };
}

export const DEFAULT_HMR_OPTIONS: Required<UserHmrConfig> = {
  ignores: [],
  host: true,
  port: 9000,
  path: '/__hmr',
  watchOptions: {
    awaitWriteFinish: 10
  }
};

export const DEFAULT_DEV_SERVER_OPTIONS: NormalizedServerConfig = {
  // TODO more server options e.g: https
  headers: {},
  port: 9000,
  https: undefined,
  protocol: 'http',
  hostname: 'localhost',
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
      : merge({}, DEFAULT_HMR_OPTIONS, { host, port }, hmrConfig || {});

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
  });
}

async function readConfigFile(
  configFilePath: string,
  logger: Logger
): Promise<UserConfig | undefined> {
  if (fs.existsSync(configFilePath)) {
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
        .join('')}}.cjs`;

      const normalizedConfig = await normalizeUserCompilationConfig(
        null,
        {
          compilation: {
            input: {
              [fileName]: configFilePath
            },
            output: {
              entryFilename: '[entryName]',
              path: outputPath,
              format: 'cjs',
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
            persistentCache: false
          },
          server: {
            hmr: false
          }
        },
        logger
      );

      const compiler = new Compiler({
        ...normalizedConfig,
        jsPlugins: [],
        rustPlugins: []
      });

      // const previousProfileEnv = process.env.FARM_PROFILE;
      // process.env.FARM_PROFILE = '';
      await compiler.compile();
      // process.env.FARM_PROFILE = previousProfileEnv;

      compiler.writeResourcesToDisk();

      const filePath = isWindows
        ? pathToFileURL(path.join(outputPath, fileName))
        : path.join(outputPath, fileName);

      try {
        // Change to vm.module of node or loaders as far as it is stable
        return (await import(filePath as string)).default;
      } finally {
        fs.unlink(filePath, () => void 0);
      }
    } else {
      const filePath = isWindows
        ? pathToFileURL(configFilePath)
        : configFilePath;
      // Change to vm.module of node or loaders as far as it is stable
      return (await import(filePath as string)).default;
    }
  }
}

export function mergeUserConfig(
  config: Record<string, any>,
  options: Record<string, any>
) {
  // The merge property can only be enabled if command line arguments are passed
  return mergeConfiguration(config, options);
}

export function mergeConfiguration(
  a: Record<string, any>,
  b: Record<string, any>
): Record<string, any> {
  const result: Record<string, any> = { ...a };
  for (const key in b) {
    if (Object.prototype.hasOwnProperty.call(b, key)) {
      const value = b[key];
      if (value == null) {
        continue;
      }
      if (isArray(value)) {
        result[key] = result[key]
          ? [...new Set([...result[key], ...value])]
          : value;
      } else if (isObject(value)) {
        result[key] = mergeConfiguration(result[key] || {}, value);
      } else {
        result[key] = value;
      }
    }
  }
  return result;
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

async function loadFileConfig(
  userConfig: ResolvedUserConfig,
  inlineOptions: FarmCLIOptions,
  logger: Logger
): Promise<ResolvedUserConfig> {
  const { configPath } = inlineOptions;
  // if configPath points to a directory, try to find a config file in it using default config
  if (fs.statSync(configPath).isDirectory()) {
    for (const name of DEFAULT_CONFIG_NAMES) {
      const resolvedPath = path.join(configPath, name);
      const config = await readConfigFile(resolvedPath, logger);
      const farmConfig = mergeUserConfig(config, inlineOptions);
      if (config) {
        userConfig = parseUserConfig(farmConfig);
        userConfig.configFilePath = resolvedPath;
        // if we found a config file, stop searching
        break;
      }
    }
  } else if (fs.statSync(configPath).isFile()) {
    const config = await readConfigFile(configPath, logger);
    const farmConfig = mergeUserConfig(config, inlineOptions);

    if (config) {
      userConfig = parseUserConfig(farmConfig);
      userConfig.configFilePath = configPath;
    }
  }

  if (userConfig.configFilePath) {
    const dependencies = await traceDependencies(userConfig.configFilePath);
    dependencies.sort();
    userConfig.configFileDependencies = dependencies;
  }
  delete userConfig.configPath;
  return userConfig;
}

async function resolveFarmPlugins(config: UserConfig) {
  const plugins = config.plugins ?? [];

  if (!plugins.length) {
    return {
      rustPlugins: [],
      jsPlugins: []
    };
  }

  const rustPlugins = [];

  const jsPlugins: JsPlugin[] = [];

  for (const plugin of plugins) {
    if (
      typeof plugin === 'string' ||
      (isArray(plugin) && typeof plugin[0] === 'string')
    ) {
      rustPlugins.push(await rustPluginResolver(plugin as string, config.root));
    } else if (isObject(plugin)) {
      convertPlugin(plugin as unknown as JsPlugin);
      jsPlugins.push(plugin as unknown as JsPlugin);
    } else if (isArray(plugin)) {
      for (const pluginNestItem of plugin as JsPlugin[]) {
        convertPlugin(pluginNestItem as JsPlugin);
        jsPlugins.push(pluginNestItem as JsPlugin);
      }
    } else {
      throw new Error(
        `plugin ${plugin} is not supported, Please pass the correct plugin type`
      );
    }
  }

  return {
    rustPlugins,
    jsPlugins
  };
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
