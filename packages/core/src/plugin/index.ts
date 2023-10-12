import type { Config } from '../../binding/index.js';
import type { UserConfig } from '../config/index.js';
import { isArray, isObject } from '../utils/share.js';
import { convertPlugin, handleVitePlugins } from './js/index.js';
import { type JsPlugin } from './type.js';
import { rustPluginResolver } from './rust/index.js';

export * from './js/index.js';
export * from './rust/index.js';

/**
 * resolvePlugins split / jsPlugins / rustPlugins
 * @param config
 */
export async function resolveAllPlugins(
  finalConfig: Config['config'],
  userConfig: UserConfig
) {
  const plugins = userConfig.plugins ?? [];
  const vitePlugins = userConfig.vitePlugins ?? [];

  if (!plugins.length && !vitePlugins?.length) {
    return {
      rustPlugins: [],
      jsPlugins: [],
      finalConfig
    };
  }

  const rustPlugins = [];

  const jsPlugins: JsPlugin[] = handleVitePlugins(
    vitePlugins,
    userConfig,
    finalConfig
  );

  for (const plugin of plugins) {
    if (
      typeof plugin === 'string' ||
      (isArray(plugin) && typeof plugin[0] === 'string')
    ) {
      rustPlugins.push(
        await rustPluginResolver(plugin as string, finalConfig.root)
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
