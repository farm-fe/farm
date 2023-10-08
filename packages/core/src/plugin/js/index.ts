import merge from 'lodash.merge';
import { Config } from '../../../binding/index.js';
import {
  type JsPlugin,
  type UserConfig,
  normalizeDevServerOptions
} from '../../index.js';
import { VITE_PLUGIN_DEFAULT_MODULE_TYPE } from './utils.js';
import { VitePluginAdapter } from './vite-plugin-adapter.js';
import { existsSync, readFileSync } from 'node:fs';

// export * from './jsPluginAdapter.js';
export { VitePluginAdapter } from './vite-plugin-adapter.js';

export function handleVitePlugins(
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
      filters = ['!node_modules'];

    if (typeof vitePluginObj === 'function') {
      const { vitePlugin: plugin, filters: f } = vitePluginObj();
      vitePlugin = plugin;
      filters = f;
    }

    if (Array.isArray(vitePlugin)) {
      for (const plugin of vitePlugin) {
        const vitePluginAdapter = new VitePluginAdapter(
          plugin as any,
          userConfig,
          filters
        );
        convertPlugin(vitePluginAdapter);
        jsPlugins.push(vitePluginAdapter);
      }
    } else {
      const vitePluginAdapter = new VitePluginAdapter(
        vitePlugin as any,
        userConfig,
        filters
      );
      convertPlugin(vitePluginAdapter);
      jsPlugins.push(vitePluginAdapter);
    }
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
