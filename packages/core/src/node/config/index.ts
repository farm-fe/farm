import fs from 'fs';
import path from 'path';
import merge from 'lodash.merge';

import { Config } from '../../../binding';
import { JsPlugin } from '../plugin';
import { RustPlugin, rustPluginResolver } from '../plugin/rustPluginResolver';

export const DEFAULT_CONFIG_NAMES = [
  'farm.config.ts',
  'farm.config.js',
  'farm.config.mjs',
];

export interface UserServerConfig {
  port: number;
  hmr: boolean | UserHmrConfig;
}

export interface UserHmrConfig {
  /** ignored watch paths of the module graph, entries of this option should be a string regexp  */
  ignores?: string[];
}

export interface UserConfig {
  /** current root of this project, default to current working directory */
  root?: string;
  /** js plugin(which is a javascript object) and rust plugin(which is string refer to a .farm file or a package) */
  plugins?: (RustPlugin | JsPlugin)[];
  /** config related to compilation */
  compilation?: Config['config'];
  /** config related to dev server */
  server?: UserServerConfig;
}

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

  if (!config.runtime) {
    config.runtime = {
      path: require.resolve('@farmfe/runtime'),
      plugins: [],
    };
  } else if (!config.runtime.path) {
    config.runtime.path = require.resolve('@farmfe/runtime');
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
    // rustPlugins: [],
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
  // if configPath points to a directory, try to find a config file in it using default config
  if (fs.statSync(configPath).isDirectory()) {
    for (const name of DEFAULT_CONFIG_NAMES) {
      const resolvedPath = path.join(configPath, name);

      if (fs.existsSync(resolvedPath)) {
        // if config is written in typescript, we need to compile it to javascript using farm first
        if (name.endsWith('.ts')) {
          // TODO
        } else {
          const config = (await import(resolvedPath)).default;
          return config;
        }
      }
    }
  }
  return {};
}

export function defineFarmConfig(userConfig: UserConfig): UserConfig {
  return userConfig;
}
