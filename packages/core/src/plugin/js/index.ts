import merge from 'lodash.merge';
import { Config } from '../../../binding/index.js';
import {
  type JsPlugin,
  normalizeDevServerOptions,
  type UserConfig
} from '../../index.js';
import {
  DEFAULT_FILTERS,
  getCssModuleType,
  VITE_PLUGIN_DEFAULT_MODULE_TYPE
} from './utils.js';
import { VitePluginAdapter } from './vite-plugin-adapter.js';
import { existsSync, readFileSync } from 'node:fs';

// export * from './jsPluginAdapter.js';
export { VitePluginAdapter } from './vite-plugin-adapter.js';

type VitePluginType = object | (() => { vitePlugin: any; filters: string[] });
type VitePluginsType = VitePluginType[];

export function handleVitePlugins(
  vitePlugins: VitePluginsType,
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
      filters = DEFAULT_FILTERS;

    if (typeof vitePluginObj === 'function') {
      const { vitePlugin: plugin, filters: f } = vitePluginObj();
      vitePlugin = plugin;
      filters = f;
    }
    processVitePlugin(vitePlugin, userConfig, filters, jsPlugins);
  }

  // if vitePlugins is not empty, append a load plugin to load file
  // this plugin is only for compatibility
  if (vitePlugins.length) {
    jsPlugins.push({
      name: 'farm:load',
      // has to be the last one
      priority: -100,
      load: {
        filters: {
          resolvedPaths: DEFAULT_FILTERS
        },
        executor: async (params) => {
          const { resolvedPath } = params;

          // skip lazy compiled module and non-exist file
          if (
            VitePluginAdapter.isFarmInternalVirtualModule(resolvedPath) ||
            !existsSync(resolvedPath)
          ) {
            return null;
          }

          const content = readFileSync(resolvedPath, 'utf-8');

          return {
            content,
            moduleType: VITE_PLUGIN_DEFAULT_MODULE_TYPE
          };
        }
      },
      transform: {
        filters: {
          resolvedPaths: DEFAULT_FILTERS,
          moduleTypes: []
        },
        executor: async (params) => {
          const { content, moduleId, moduleType, resolvedPath } = params;

          // skip lazy compiled module and non-exist file
          if (VitePluginAdapter.isFarmInternalVirtualModule(resolvedPath)) {
            return null;
          }
          const cssModules = finalConfig?.css?.modules?.paths ?? [
            '\\.module\\.(css|less|sass|scss)$'
          ];
          // skip css module because it will be handled by Farm
          const isCssModules = cssModules.some((reg) =>
            new RegExp(reg).test(moduleId)
          );

          // treat all scss/less/.etc lang as css
          // plugin should handle css module by itself
          if (getCssModuleType(moduleId) && !isCssModules) {
            return {
              content,
              moduleType: 'css'
            };
          }

          return {
            content,
            moduleType
          };
        }
      }
    });
  }

  return jsPlugins;
}

export function processVitePlugin(
  vitePlugin: VitePluginType,
  userConfig: UserConfig,
  filters: string[],
  jsPlugins: JsPlugin[]
) {
  const processPlugin = (plugin: any) => {
    const vitePluginAdapter = new VitePluginAdapter(
      plugin as any,
      userConfig,
      filters
    );
    convertPlugin(vitePluginAdapter);
    jsPlugins.push(vitePluginAdapter);
  };

  if (Array.isArray(vitePlugin)) {
    vitePlugin.forEach((plugin) => processPlugin(plugin));
  } else {
    processPlugin(vitePlugin);
  }
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

  if (plugin.load?.filters?.resolvedPaths?.length) {
    if (process.platform === 'win32') {
      // replace / to \
      plugin.load.filters.resolvedPaths = plugin.load.filters.resolvedPaths.map(
        (item) => item.replaceAll('/', '\\\\')
      );
    }
  }

  if (plugin.transform?.filters?.resolvedPaths?.length) {
    if (process.platform === 'win32') {
      // replace / to \
      plugin.transform.filters.resolvedPaths =
        plugin.transform.filters.resolvedPaths.map((item) =>
          item.replaceAll('/', '\\\\')
        );
    }
  }
}
