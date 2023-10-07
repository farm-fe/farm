import merge from 'lodash.merge';
import { Config } from '../../../binding/index.js';
import {
  isArray,
  rustPluginResolver,
  type JsPlugin,
  type UserConfig,
  convertPlugin,
  normalizeDevServerOptions
} from '../../index.js';
import { VITE_PLUGIN_DEFAULT_MODULE_TYPE, isObject } from './utils.js';
import { VitePluginAdapter } from './vite-plugin-adapter.js';
import { existsSync, readFileSync } from 'node:fs';

// export * from './jsPluginAdapter.js';
export { VitePluginAdapter } from './vite-plugin-adapter.js';

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

function handleVitePlugins(
  vitePlugins: (object | (() => { vitePlugin: any; filters: string[] }))[],
  userConfig: UserConfig,
  finalConfig: Config['config']
): JsPlugin[] {
  const jsPlugins: JsPlugin[] = [];

  if (vitePlugins.length) {
    userConfig = merge({}, userConfig, {
      compilation: finalConfig,
      server: normalizeDevServerOptions(userConfig.server, finalConfig.mode)
    });
  }

  for (const vitePluginObj of vitePlugins) {
    let vitePlugin = vitePluginObj,
      filters = ['.*'];

    if (typeof vitePluginObj === 'function') {
      const { vitePlugin: plugin, filters: f } = vitePluginObj();
      vitePlugin = plugin;
      filters = f;
    }

    const vitePluginAdapter = new VitePluginAdapter(
      vitePlugin as any,
      userConfig,
      filters
    );
    convertPlugin(vitePluginAdapter);
    jsPlugins.push(vitePluginAdapter);
  }

  // if vitePlugins is not empty, append a load plugin to load files as js
  if (vitePlugins.length) {
    jsPlugins.push({
      name: 'farm:load',
      // has to be the last one
      priority: 0,
      load: {
        filters: {
          resolvedPaths: ['.*']
        },
        executor: async (params) => {
          const { resolvedPath } = params;

          if (!existsSync(resolvedPath)) {
            console.log('load', params);
          }

          const content = readFileSync(resolvedPath, 'utf-8');

          return {
            content,
            moduleType: VITE_PLUGIN_DEFAULT_MODULE_TYPE
          };
        }
      }
    });
  }

  return jsPlugins;
}
