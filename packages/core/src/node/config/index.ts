import path from 'path';
import { Config } from '../../../binding';
import { Plugin } from '../plugin';

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
  /** js plugin(which is a javascript object) and wasm plugin(which is string refer to a wasm file or a package) */
  plugins?: (string | Plugin)[];
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
  const config: Config['config'] = {};

  for (const key of Object.keys(userConfig.compilation ?? {})) {
    config[key] = userConfig.compilation[key];
  }

  if (!config.runtime) {
    config.runtime = {
      path: require.resolve('@farmfe/runtime'),
      plugins: [],
    };
  }

  const normalizedConfig: Config = {
    config,
    rustPlugins: [
      // path.join(
      //   __dirname,
      //   '../../../../../target/release/libfarmfe_plugin_sass.so'
      // ),
      // path.join(
      //   __dirname,
      //   '../../../../../target/release/libfarmfe_plugin_sass.dylib'
      // ),
    ],
    // rustPlugins: [],
    jsPlugins: [
      // {
      //   name: 'js-plugin',
      //   priority: 10,
      //   resolve: {
      //     filters: {
      //       importers: [],
      //       sources: ['from_js_plugin'],
      //     },
      //     executor: async (param, context, hook_context) => {
      //       console.log(param, context, hook_context);
      //       if (!hook_context.caller) {
      //         const resolved = await context.resolve(
      //           {
      //             ...param,
      //             source: './from_js_plugin',
      //           },
      //           {
      //             meta: hook_context.meta,
      //             caller: 'js-plugin',
      //           }
      //         );
      //         console.log('call internal resolve in js', resolved);
      //         resolved.id += '.js-plugin';
      //         return resolved;
      //       }
      //     },
      //   },
      // },
    ],
    // jsPlugins: [],
  };

  return normalizedConfig;
}

/**
 * Resolve and load user config from the specified path
 * @param configPath
 */
export function resolveUserConfig(configPath: string): UserConfig {
  return {};
}

export function defineFarmConfig(userConfig: UserConfig): UserConfig {
  return userConfig;
}
