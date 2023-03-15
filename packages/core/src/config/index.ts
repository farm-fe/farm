import module from 'node:module';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';

import merge from 'lodash.merge';
import chalk from 'chalk';

import { bindingPath, Config } from '../../binding/index.js';
import { JsPlugin } from '../plugin/index.js';
import { rustPluginResolver } from '../plugin/rustPluginResolver.js';
import {
  NormalizedServerConfig,
  UserConfig,
  UserHmrConfig,
  UserServerConfig,
} from './types.js';
import { Logger } from '../logger.js';
import { pathToFileURL } from 'node:url';

export * from './types.js';
export const DEFAULT_CONFIG_NAMES = [
  'farm.config.ts',
  'farm.config.js',
  'farm.config.mjs',
];

/**
 * Normalize user config and transform it to rust compiler compatible config
 * @param config
 * @returns resolved config that parsed to rust compiler
 */
export async function normalizeUserCompilationConfig(
  userConfig: UserConfig,
  mode: 'development' | 'production' = 'production'
): Promise<Config> {
  const config: Config['config'] = merge(
    {
      input: {
        index: './index.html',
      },
      output: {
        path: './dist',
      },
    },
    userConfig.compilation
  );
  config.coreLibPath = bindingPath;
  const require = module.createRequire(import.meta.url);
  const hmrClientPluginPath = require.resolve('@farmfe/runtime-plugin-hmr');

  if (!config.runtime) {
    config.runtime = {
      path: require.resolve('@farmfe/runtime'),
      plugins: [],
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

  if (config.lazyCompilation === undefined) {
    if (mode === 'development') {
      config.lazyCompilation = true;
    } else {
      config.lazyCompilation = false;
    }
  }

  if (config.mode === undefined) {
    config.mode = mode;
  }

  if (config.mode === 'production') {
    if (!config.output?.filename) {
      config.output.filename = '[resourceName].[contentHash].[ext]';
    }
    if (!config.output?.assetsFilename) {
      config.output.assetsFilename = '[resourceName].[contentHash].[ext]';
    }
  }

  const normalizedDevServerConfig = normalizeDevServerOptions(
    userConfig.server
  );

  if (
    Array.isArray(config.runtime.plugins) &&
    normalizedDevServerConfig.hmr &&
    !config.runtime.plugins.includes(hmrClientPluginPath)
  ) {
    config.runtime.plugins.push(hmrClientPluginPath);
  }

  // we should not deep merge compilation.input
  if (userConfig.compilation?.input) {
    config.input = userConfig.compilation.input;
  }

  if (!config.root) {
    config.root = userConfig.root ?? process.cwd();
  }

  const plugins = userConfig.plugins ?? [];
  const rustPlugins = [];
  const jsPlugins = [];

  for (const plugin of plugins) {
    if (typeof plugin === 'string' || Array.isArray(plugin)) {
      rustPlugins.push(await rustPluginResolver(plugin, config.root as string));
    } else if (typeof plugin === 'object') {
      jsPlugins.push(plugin as JsPlugin);
    }
  }

  const normalizedConfig: Config = {
    config,
    rustPlugins,
    jsPlugins,
  };

  return normalizedConfig;
}

export const DEFAULT_HMR_OPTIONS: Required<UserHmrConfig> = {
  ignores: [],
  host: 'localhost',
  port: 9801,
};

export const DEFAULT_DEV_SERVER_OPTIONS: NormalizedServerConfig = {
  port: 9000,
  https: false,
  // http2: false,
  hmr: DEFAULT_HMR_OPTIONS,
};

export function normalizeDevServerOptions(
  options?: UserServerConfig
): NormalizedServerConfig {
  if (!options) {
    return DEFAULT_DEV_SERVER_OPTIONS;
  }

  if (options.hmr === true) {
    options.hmr = DEFAULT_HMR_OPTIONS;
  }

  return merge({}, DEFAULT_DEV_SERVER_OPTIONS, options);
}

/**
 * Resolve and load user config from the specified path
 * @param configPath
 */
export async function resolveUserConfig(
  configPath: string,
  logger: Logger
): Promise<UserConfig> {
  if (!path.isAbsolute(configPath)) {
    throw new Error('configPath must be an absolute path');
  }

  let userConfig: UserConfig = {};
  let root: string = process.cwd();

  // if configPath points to a directory, try to find a config file in it using default config
  if (fs.statSync(configPath).isDirectory()) {
    root = configPath;

    for (const name of DEFAULT_CONFIG_NAMES) {
      const resolvedPath = path.join(configPath, name);
      const config = await readConfigFile(resolvedPath, logger);

      if (config) {
        userConfig = config;
        // if we found a config file, stop searching
        break;
      }
    }
  } else if (fs.statSync(configPath).isFile()) {
    root = path.dirname(configPath);

    const config = await readConfigFile(configPath, logger);

    if (config) {
      userConfig = config;
    }
  }

  if (!userConfig.root) {
    userConfig.root = root;
  }

  return userConfig;
}

async function readConfigFile(
  resolvedPath: string,
  logger: Logger
): Promise<UserConfig | undefined> {
  if (fs.existsSync(resolvedPath)) {
    logger.info(`Using config file at ${chalk.green(resolvedPath)}`);
    // if config is written in typescript, we need to compile it to javascript using farm first
    if (resolvedPath.endsWith('.ts')) {
      const Compiler = (await import('../compiler/index.js')).Compiler;
      const outputPath = path.join(
        os.tmpdir(),
        'farmfe',
        Date.now().toString()
      );
      const fileName = 'farm.config.bundle.mjs';
      const normalizedConfig = await normalizeUserCompilationConfig({
        compilation: {
          input: {
            config: resolvedPath,
          },
          output: {
            filename: fileName,
            path: outputPath,
          },
          external: [
            ...module.builtinModules,
            ...module.builtinModules.map((m) => `node:${m}`),
          ],
          partialBundling: {
            moduleBuckets: [
              {
                name: fileName,
                test: ['.+'],
              },
            ],
          },
          sourcemap: false,
        },
        server: {
          hmr: false,
        },
      });
      const compiler = new Compiler(normalizedConfig);
      await compiler.compile();
      compiler.writeResourcesToDisk();

      const filePath = path.join(outputPath, fileName);
      // Change to vm.module of node or loaders as far as it is stable
      if (process.platform === 'win32') {
        return (await import(pathToFileURL(filePath).toString())).default;
      } else {
        return (await import(filePath)).default;
      }
    } else {
      // Change to vm.module of node or loaders as far as it is stable
      if (process.platform === 'win32') {
        return (await import(pathToFileURL(resolvedPath).toString())).default;
      } else {
        return (await import(resolvedPath)).default;
      }
    }
  }
}
