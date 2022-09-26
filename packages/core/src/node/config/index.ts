import merge from 'lodash.merge';

import { Config } from '../../../binding';
import { JsPlugin } from '../plugin';
import { RustPlugin, rustPluginResolver } from '../plugin/rustPluginResolver';

export interface UserServerConfig {
  port: number;
}

export interface UserWatcherConfig {
  /** ignored watch paths of the module graph, entry of this option is a string regexp  */
  ignores: string[];
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
  watcher?: UserWatcherConfig;
}

/**
 * Normalize user config and transform it to rust compiler compatible config
 * @param config
 * @returns resolved config that parsed to rust compiler
 */
export function normalizeUserCompilationConfig(userConfig: UserConfig): Config {
  const config: Config['config'] = merge({}, userConfig);

  if (!config.runtime) {
    config.runtime = {
      path: require.resolve('@farmfe/runtime'),
      plugins: [],
    };
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
export function resolveUserConfig(_configPath: string): UserConfig {
  return {};
}

export function defineFarmConfig(userConfig: UserConfig): UserConfig {
  return userConfig;
}
