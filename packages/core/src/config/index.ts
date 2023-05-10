import module from 'node:module';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import readline from 'node:readline';

import merge from 'lodash.merge';
import chalk from 'chalk';

import { bindingPath, Config } from '../../binding/index.js';
import { JsPlugin } from '../plugin/index.js';
import { rustPluginResolver } from '../plugin/rustPluginResolver.js';
import {
  FarmCLIOptions,
  NormalizedServerConfig,
  UserConfig,
  UserHmrConfig,
  UserServerConfig
} from './types.js';
import { Logger } from '../logger.js';
import { pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';
import { parseUserConfig } from './schema.js';

export * from './types.js';
export const DEFAULT_CONFIG_NAMES = [
  'farm.config.ts',
  'farm.config.js',
  'farm.config.mjs'
];

type CompilationMode = 'development' | 'production';

/**
 * Normalize user config and transform it to rust compiler compatible config
 * @param config
 * @returns resolved config that parsed to rust compiler
 */
export async function normalizeUserCompilationConfig(
  userConfig: UserConfig,
  mode: CompilationMode = 'production'
): Promise<Config> {
  const config: Config['config'] = merge(
    {
      input: {
        index: './index.html'
      },
      output: {
        path: './dist'
      }
    },
    userConfig.compilation
  );
  config.coreLibPath = bindingPath;
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

  const normalizedDevServerConfig = normalizeDevServerOptions(
    userConfig.server,
    mode
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

  if (config.treeShaking === undefined) {
    if (mode === 'production') {
      config.treeShaking = true;
    } else {
      config.treeShaking = false;
    }
  }

  if (config.minify === undefined) {
    if (mode === 'production') {
      config.minify = true;
    } else {
      config.minify = false;
    }
  }

  if (config.presetEnv === undefined) {
    if (mode === 'production') {
      config.presetEnv = true;
    } else {
      config.presetEnv = false;
    }
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

  let finalConfig = config;
  // call user config hooks
  for (const jsPlugin of jsPlugins) {
    finalConfig = (await jsPlugin.config?.(finalConfig)) ?? finalConfig;
  }

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
  port: 9801
};

export const DEFAULT_DEV_SERVER_OPTIONS: NormalizedServerConfig = {
  port: 9000,
  https: false,
  // http2: false,
  proxy: {},
  hmr: DEFAULT_HMR_OPTIONS,
  strictPort: false
};

export function normalizeDevServerOptions(
  options: UserServerConfig | undefined,
  mode: CompilationMode
): NormalizedServerConfig {
  return merge({}, DEFAULT_DEV_SERVER_OPTIONS, options, {
    hmr:
      mode === 'production'
        ? false
        : options?.hmr !== false
        ? DEFAULT_HMR_OPTIONS
        : options.hmr
  });
}

/**
 * Resolve and load user config from the specified path
 * @param configPath
 */
export async function resolveUserConfig(
  options: FarmCLIOptions,
  logger: Logger,
  command: 'start' | 'build'
): Promise<UserConfig> {
  const { configPath } = options;

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
      if (command === 'start') {
        clearScreen();
      }
      const config = await readConfigFile(resolvedPath, logger);

      // The merge property can only be enabled if command line arguments are passed
      const filterOptions = cleanConfig(options);
      if (!isEmptyObject(filterOptions)) {
        mergeConfig(config, options, command);
      }

      if (config) {
        userConfig = parseUserConfig(config);
        // if we found a config file, stop searching
        break;
      }
    }
  } else if (fs.statSync(configPath).isFile()) {
    root = path.dirname(configPath);
    if (command === 'start') {
      clearScreen();
    }
    const config = await readConfigFile(configPath, logger);

    const filterOptions = cleanConfig(options);
    if (!isEmptyObject(filterOptions)) {
      mergeConfig(config, options, command);
    }

    if (config) {
      userConfig = parseUserConfig(config);
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
      const hash = createHash('md5');
      const outputPath = path.join(
        os.tmpdir(),
        'farmfe',
        hash.update(resolvedPath).digest('hex')
      );
      const fileName = 'farm.config.bundle.mjs';
      const normalizedConfig = await normalizeUserCompilationConfig({
        compilation: {
          input: {
            config: resolvedPath
          },
          output: {
            filename: fileName,
            path: outputPath,
            targetEnv: 'node'
          },
          external: [
            ...module.builtinModules.map((m) => `^${m}$`),
            ...module.builtinModules.map((m) => `^node:${m}$`)
          ],
          partialBundling: {
            moduleBuckets: [
              {
                name: fileName,
                test: ['.+']
              }
            ]
          },
          sourcemap: false,
          treeShaking: false,
          minify: false,
          presetEnv: false
        },
        server: {
          hmr: false
        }
        // plugins: [
        //   {
        //     name: 'farm-plugin-external',
        //     resolve: {
        //       filters: {
        //         importers: ['.+'],
        //         sources: ['.+']
        //       },
        //       executor: (param) => {
        //         // external all non-relative and non-absolute imports
        //         if (
        //           path.isAbsolute(param.source) ||
        //           param.source.startsWith('.') ||
        //           param.source.startsWith('@swc/helpers')
        //         ) {
        //           return null;
        //         } else {
        //           console.log('external', param.source);
        //           return {
        //             resolvedPath: 'external-path',
        //             external: true,
        //             sideEffects: false,
        //             query: [],
        //             meta: {}
        //           };
        //         }
        //       }
        //     }
        //   }
        // ]
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

export function mergeConfig(
  config: UserConfig,
  options: FarmCLIOptions,
  command: 'start' | 'build'
) {
  // merge options
  if (command === 'start') {
    mergeServerOptions(config, options);
  }

  if (command === 'build') {
    mergeBuildOptions(config, options);
  }
}

export function cleanConfig(config: FarmCLIOptions): FarmCLIOptions {
  delete config.configPath;
  delete config.config;
  delete config.outDir;
  delete config.strictPort;
  delete config.open;
  return config;
}

// TODO optimizing merge methods
export function mergeServerOptions(
  config: UserConfig,
  options: FarmCLIOptions
) {
  config.server = merge(config.server, options);
}

export function mergeBuildOptions(config: UserConfig, options: FarmCLIOptions) {
  if (options.outDir) {
    config.compilation.output.path = options.outDir;
  }
  config.compilation = merge(config.compilation, options);
}

export function clearScreen() {
  const repeatCount = process.stdout.rows - 2;
  const blank = repeatCount > 0 ? '\n'.repeat(repeatCount) : '';
  console.log(blank);
  readline.cursorTo(process.stdout, 0, 0);
  readline.clearScreenDown(process.stdout);
}

export function isEmptyObject<T extends object>(obj: T): boolean {
  return Reflect.ownKeys(obj).length === 0;
}
