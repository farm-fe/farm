import module from 'node:module';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

import merge from 'lodash.merge';
import chalk from 'chalk';

import { bindingPath, Config } from '../../binding/index.js';
import { JsPlugin } from '../plugin/index.js';
import { DevServer } from '../server/index.js';
import { rustPluginResolver } from '../plugin/rustPluginResolver.js';
import { parseUserConfig } from './schema.js';
import {
  clearScreen,
  isArray,
  isObject,
  Logger,
  normalizePath
} from '../utils/index.js';

import type {
  FarmCLIOptions,
  NormalizedServerConfig,
  UserConfig,
  UserHmrConfig,
  UserServerConfig
} from './types.js';

import { CompilationMode, loadEnv } from './env.js';
import { __FARM_GLOBAL__ } from './_global.js';
import { importFresh } from '../utils/share.js';

export * from './types.js';
export const DEFAULT_CONFIG_NAMES = [
  'farm.config.ts',
  'farm.config.js',
  'farm.config.mjs'
];
export const urlRegex = /^(https?:)?\/\/([^/]+)/;

/**
 * Normalize user config and transform it to rust compiler compatible config
 * @param config
 * @returns resolved config that parsed to rust compiler
 */
export async function normalizeUserCompilationConfig(
  userConfig: UserConfig,
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
  const config: Config['config'] = merge(
    {
      input: {
        index: './index.html'
      },
      output: {
        path: './dist'
      }
    },
    compilation
  );
  config.mode = config.mode ?? mode;
  const isProduction = config.mode === 'production';
  const isDevelopment = config.mode === 'development';

  config.coreLibPath = bindingPath;

  const resolvedEnvPath = envDir ? envDir : resolvedRootPath;

  const userEnv = loadEnv(
    compilation?.mode ?? mode,
    resolvedEnvPath,
    envPrefix
  );

  config.env = {
    ...userEnv,
    NODE_ENV: process.env.NODE_ENV || mode
  };

  config.define = Object.assign(
    {},
    config?.define,
    Object.keys(config.env).reduce((env: any, key) => {
      env[`process.env.${key}`] = config.env[key];
      return env;
    }, {})
  );

  const require = module.createRequire(import.meta.url);
  const hmrClientPluginPath = require.resolve('@farmfe/runtime-plugin-hmr');

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

  if (config.lazyCompilation === undefined) {
    if (isDevelopment) {
      config.lazyCompilation = true;
    } else {
      config.lazyCompilation = false;
    }
  }

  if (config.mode === undefined) {
    config.mode = mode;
  }

  if (isProduction) {
    if (!config.output) {
      config.output = {};
    }
    if (!config.output.filename) {
      config.output.filename = '[resourceName].[contentHash].[ext]';
    }
    if (!config.output.assetsFilename) {
      config.output.assetsFilename = '[resourceName].[contentHash].[ext]';
    }
  }

  if (config?.output?.targetEnv === 'node') {
    config.external = [
      ...(config.external ?? []),
      ...module.builtinModules.map((m) => `^${m}($|/)`),
      ...module.builtinModules.map((m) => `^node:${m}($|/)`)
    ];
  }

  // TODO resolve other server port
  const normalizedDevServerConfig = normalizeDevServerOptions(server, mode);

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

  if (!config.root) {
    config.root = root ?? process.cwd();
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

  if (config.presetEnv === undefined) {
    if (isProduction) {
      config.presetEnv = true;
    } else {
      config.presetEnv = false;
    }
  }
  const { jsPlugins, rustPlugins, finalConfig } = await resolveAllPlugins(
    config,
    userConfig
  );

  const normalizedConfig: Config = {
    config: finalConfig,
    rustPlugins,
    jsPlugins
  };

  return normalizedConfig;
}

export const DEFAULT_HMR_OPTIONS: Required<UserHmrConfig> = {
  ignores: [],
  host: 'localhost',
  port: 9000,
  path: '/__hmr',
  watchOptions: {
    awaitWriteFinish: 10
  }
};

export const DEFAULT_DEV_SERVER_OPTIONS: NormalizedServerConfig = {
  headers: {},
  port: 9000,
  https: false,
  protocol: 'http',
  hostname: 'localhost',
  // http2: false,
  host: 'localhost',
  proxy: {},
  hmr: DEFAULT_HMR_OPTIONS,
  open: false,
  strictPort: false,
  cors: false,
  spa: true,
  plugins: [],
  writeToDisk: false
};

export function normalizeDevServerOptions(
  options: UserServerConfig | undefined,
  mode: string
): NormalizedServerConfig {
  const { host, port, hmr: hmrConfig } = options || {};
  const isProductionMode = mode === 'production';
  const hmr =
    isProductionMode || hmrConfig === false
      ? false
      : merge({}, DEFAULT_HMR_OPTIONS, { host, port }, hmrConfig || {});

  return merge({}, DEFAULT_DEV_SERVER_OPTIONS, options, { hmr });
}

/**
 * Resolve and load user config from the specified path
 * @param configPath
 */
export async function resolveUserConfig(
  inlineOptions: FarmCLIOptions,
  logger: Logger
): Promise<UserConfig> {
  let userConfig: UserConfig = {};
  let root: string = process.cwd();

  const { configPath } = inlineOptions;
  if (
    inlineOptions.clearScreen &&
    __FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__
  ) {
    clearScreen();
  }

  if (!path.isAbsolute(configPath)) {
    throw new Error('configPath must be an absolute path');
  }

  // if configPath points to a directory, try to find a config file in it using default config
  if (fs.statSync(configPath).isDirectory()) {
    root = configPath;

    for (const name of DEFAULT_CONFIG_NAMES) {
      const resolvedPath = path.join(configPath, name);
      const config = await readConfigFile(resolvedPath, logger);
      const farmConfig = mergeUserConfig(config, inlineOptions);
      if (config) {
        userConfig = parseUserConfig(farmConfig);
        userConfig.resolveConfigPath = resolvedPath;
        // if we found a config file, stop searching
        break;
      }
    }
  } else if (fs.statSync(configPath).isFile()) {
    root = path.dirname(configPath);
    const config = await readConfigFile(configPath, logger);
    const farmConfig = mergeUserConfig(config, inlineOptions);

    if (config) {
      userConfig = parseUserConfig(farmConfig);
      userConfig.resolveConfigPath = configPath;
    }
  }

  if (!userConfig.root) {
    userConfig.root = root;
  }

  // check port availability: auto increment the port if a conflict occurs
  await DevServer.resolvePortConflict(userConfig, logger);
  // Save variables are used when restarting the service
  const config = filterUserConfig(userConfig, inlineOptions);
  return config;
}

async function readConfigFile(
  configFilePath: string,
  logger: Logger
): Promise<UserConfig | undefined> {
  if (fs.existsSync(configFilePath)) {
    __FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ &&
      logger.info(`Using config file at ${chalk.green(configFilePath)}`);
    // if config is written in typescript, we need to compile it to javascript using farm first
    if (configFilePath.endsWith('.ts')) {
      const Compiler = (await import('../compiler/index.js')).Compiler;
      const outputPath = path.join(
        path.dirname(configFilePath),
        'node_modules',
        '.farm'
      );
      const fileName = 'farm.config.bundle.mjs';
      const normalizedConfig = await normalizeUserCompilationConfig(
        {
          compilation: {
            input: {
              [fileName]: configFilePath
            },
            output: {
              entryFilename: '[entryName]',
              path: outputPath,
              targetEnv: 'node'
            },
            external: [
              ...module.builtinModules.map((m) => `^${m}$`),
              ...module.builtinModules.map((m) => `^node:${m}$`),
              '^[^./].*'
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
            lazyCompilation: false
          },
          server: {
            hmr: false
          }
        },
        logger
      );

      const compiler = new Compiler(normalizedConfig);
      await compiler.compile();
      compiler.writeResourcesToDisk();

      const filePath = path.join(outputPath, fileName);

      // Change to vm.module of node or loaders as far as it is stable
      return await importFresh(filePath);
    } else {
      // Change to vm.module of node or loaders as far as it is stable
      return await importFresh(configFilePath);
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
        result[key] = result[key] ? [...result[key], ...value] : value;
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
 *
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

export function convertPlugin(plugin: JsPlugin): void {
  if (
    plugin.transform &&
    !plugin.transform.filters?.moduleTypes &&
    !plugin.transform.filters?.resolvedPaths
  ) {
    throw new Error(
      `transform hook of plugin ${plugin.name} must have at least one filter(like moduleTypes or resolvedPaths)`
    );
  }
  if (plugin.transform) {
    if (!plugin.transform.filters.moduleTypes) {
      plugin.transform.filters.moduleTypes = [];
    } else if (!plugin.transform.filters.resolvedPaths) {
      plugin.transform.filters.resolvedPaths = [];
    }
  }
}

/**
 * resolvePlugins split / jsPlugins / rustPlugins
 * @param config
 */
export async function resolveAllPlugins(
  finalConfig: Config['config'],
  userConfig: UserConfig
) {
  const plugins = userConfig.plugins ?? [];
  if (!plugins.length) {
    return {
      rustPlugins: [],
      jsPlugins: [],
      finalConfig
    };
  }
  const rustPlugins = [];
  const jsPlugins: JsPlugin[] = [];

  for (const plugin of plugins) {
    if (
      typeof plugin === 'string' ||
      (isArray(plugin) && typeof plugin[0] === 'string')
    ) {
      rustPlugins.push(await rustPluginResolver(plugin, finalConfig.root));
    } else if (isObject(plugin)) {
      convertPlugin(plugin as unknown as JsPlugin);
      jsPlugins.push(plugin as unknown as JsPlugin);
    } else if (isArray(plugin)) {
      for (const pluginNestItem of plugin) {
        convertPlugin(pluginNestItem as JsPlugin);
        jsPlugins.push(pluginNestItem as JsPlugin);
      }
    } else {
      throw new Error(
        `plugin ${plugin} is not supported, Please pass the correct plugin type`
      );
    }
  }
  // call user config hooks
  for (const jsPlugin of jsPlugins) {
    finalConfig = (await jsPlugin.config?.(finalConfig)) ?? finalConfig;
  }
  return {
    rustPlugins,
    jsPlugins,
    finalConfig
  };
}

export function filterUserConfig(
  userConfig: UserConfig,
  inlineConfig: FarmCLIOptions
): UserConfig {
  userConfig.inlineConfig = inlineConfig;
  delete userConfig.configPath;
  return userConfig;
}
