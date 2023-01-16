import module from 'node:module';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';

import merge from 'lodash.merge';

import { Config } from '../../../binding/index.js';
import { JsPlugin } from '../plugin/index.js';
import { rustPluginResolver } from '../plugin/rustPluginResolver.js';
import { UserConfig } from './types.js';

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
export function normalizeUserCompilationConfig(userConfig: UserConfig): Config {
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
  const require = module.createRequire(import.meta.url);

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
      rustPlugins.push(rustPluginResolver(plugin, config.root as string));
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

/**
 * Resolve and load user config from the specified path
 * @param configPath
 */
export async function resolveUserConfig(
  configPath: string
): Promise<UserConfig> {
  if (!path.isAbsolute(configPath)) {
    throw new Error('configPath must be an absolute path');
  }

  // if configPath points to a directory, try to find a config file in it using default config
  if (fs.statSync(configPath).isDirectory()) {
    for (const name of DEFAULT_CONFIG_NAMES) {
      const resolvedPath = path.join(configPath, name);
      const config = await resolveConfigFile(resolvedPath);

      if (config) {
        return config;
      }
    }
  } else if (fs.statSync(configPath).isFile()) {
    const config = await resolveConfigFile(configPath);

    if (config) {
      return config;
    }
  }

  return {};
}

async function resolveConfigFile(
  resolvedPath: string
): Promise<UserConfig | undefined> {
  if (fs.existsSync(resolvedPath)) {
    // if config is written in typescript, we need to compile it to javascript using farm first
    if (resolvedPath.endsWith('.ts')) {
      const Compiler = (await import('../compiler/index.js')).Compiler;
      const compiler = new Compiler({
        compilation: {
          input: {
            config: resolvedPath,
          },
        },
      });
      await compiler.compile();
      const resources = compiler.resources();
      // should only emit one config file bundled with all dependencies
      const configCode = Buffer.from(Object.values(resources)[0]).toString();
      // Change to vm.module of node or loaders as soon as it is stable
      const filePath = path.join(
        os.tmpdir(),
        'farmfe',
        `temp-config-${Date.now()}.mjs`
      );
      fs.mkdirSync(path.dirname(filePath), { recursive: true });
      fs.writeFileSync(filePath, configCode);
      const config = (await import(filePath)).default;
      return config;
    } else {
      const config = (await import(resolvedPath)).default;
      return config;
    }
  }
}
