import { isArray, isObject } from '../utils/index.js';
import { convertPlugin, handleVitePlugins } from './js/index.js';
import { rustPluginResolver } from './rust/index.js';
// import { mergeConfiguration } from '../config/index.js';

import type { JsPlugin } from './type.js';
import type { Config } from '../../binding/index.js';
import type { UserConfig } from '../config/index.js';

export * from './js/index.js';
export * from './rust/index.js';

/**
 * resolvePlugins split / jsPlugins / rustPlugins
 * @param config
 */
export async function resolveAllPlugins(
  resolvedConfig: Config['config'],
  userConfig: UserConfig
) {
  const plugins = userConfig.plugins ?? [];
  const vitePlugins = (userConfig.vitePlugins ?? []).filter(Boolean);

  if (!plugins.length && !vitePlugins?.length) {
    return {
      rustPlugins: [],
      jsPlugins: [],
      resolvedConfig
    };
  }

  const rustPlugins = [];

  const vitePluginAdapters: JsPlugin[] = handleVitePlugins(
    vitePlugins,
    userConfig,
    resolvedConfig
  );
  // console.log(vitePluginAdapters);

  const jsPlugins: JsPlugin[] = [];

  for (const plugin of plugins) {
    if (
      typeof plugin === 'string' ||
      (isArray(plugin) && typeof plugin[0] === 'string')
    ) {
      rustPlugins.push(
        await rustPluginResolver(plugin as string, resolvedConfig?.root)
      );
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
  // vite plugins execute after farm plugins by default.
  jsPlugins.push(...vitePluginAdapters);
  // const config = await resolveConfigHook(resolvedConfig, jsPlugins);
  // console.log(config);

  // call user config hooks
  for (const jsPlugin of jsPlugins) {
    resolvedConfig =
      (await jsPlugin.config?.(resolvedConfig)) ?? resolvedConfig;
  }

  return {
    rustPlugins,
    jsPlugins,
    resolvedConfig
  };
}

// export function resolvePlugins(userConfig: UserConfig) {
// }

// async function resolveConfigHook(
//   config: UserConfig,
//   plugins: JsPlugin[]
// ): Promise<UserConfig> {
//   let conf = config;

//   for (const p of getSortedPlugins(plugins)) {
//     const hook = p.config;
//     if (hook) {
//       const res = await hook(conf);
//       if (res) {
//         conf = mergeConfiguration(conf, res);
//       }
//     }
//   }

//   return conf;
// }

// export function getSortedPlugins(plugins: readonly JsPlugin[]): JsPlugin[] {
//   const priorityPre: JsPlugin[] = [];
//   const normal: JsPlugin[] = [];
//   const priorityPost: JsPlugin[] = [];
//   for (const plugin of plugins) {
//     if (plugin) {
//       if (typeof plugin === 'object') {
//         if (plugin.priority > 99) {
//           priorityPre.push(plugin);
//           continue;
//         }
//         if (plugin.priority < 99) {
//           priorityPost.push(plugin);
//           continue;
//         }
//       }
//       normal.push(plugin);
//     }
//   }

//   return [...priorityPre, ...normal, ...priorityPost] as JsPlugin[];
// }
