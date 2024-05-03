import { existsSync, readFileSync } from 'node:fs';
import { isAbsolute } from 'node:path';
import { CompilationMode } from '../../config/env.js';
import {
  type JsPlugin,
  Logger,
  type UserConfig,
  normalizeDevServerOptions
} from '../../index.js';
import merge from '../../utils/merge.js';
import { resolveAsyncPlugins } from '../index.js';
import {
  DEFAULT_FILTERS,
  VITE_PLUGIN_DEFAULT_MODULE_TYPE,
  getCssModuleType
} from './utils.js';
import { VitePluginAdapter } from './vite-plugin-adapter.js';

// export * from './jsPluginAdapter.js';
export { VitePluginAdapter } from './vite-plugin-adapter.js';

type VitePluginType = object | (() => { vitePlugin: any; filters: string[] });
type VitePluginsType = VitePluginType[];

export async function handleVitePlugins(
  vitePlugins: VitePluginsType,
  userConfig: UserConfig,
  logger: Logger,
  mode: CompilationMode
): Promise<JsPlugin[]> {
  const jsPlugins: JsPlugin[] = [];
  const filtersUnion = new Set<string>();

  if (vitePlugins.length) {
    userConfig = merge({}, userConfig, {
      compilation: userConfig.compilation,
      server: normalizeDevServerOptions(
        userConfig.server,
        userConfig.compilation?.mode ?? mode
      )
    });
  }
  const flatVitePlugins = await resolveAsyncPlugins(vitePlugins);

  for (const vitePluginObj of flatVitePlugins) {
    let vitePlugin = vitePluginObj,
      filters = DEFAULT_FILTERS;

    if (typeof vitePluginObj === 'function') {
      const { vitePlugin: plugin, filters: f } = vitePluginObj();
      vitePlugin = plugin;
      filters = f;
    }
    filters?.forEach((filter) => filtersUnion.add(filter));
    processVitePlugin(vitePlugin, userConfig, filters, jsPlugins, logger, mode);
  }

  const resolvedPaths = Array.from(filtersUnion).map(normalizeFilterPath);
  // if vitePlugins is not empty, append a load plugin to load file
  // this plugin is only for compatibility
  if (vitePlugins.length) {
    jsPlugins.push({
      name: 'farm:load',
      // has to be the last one
      priority: -100,
      load: {
        filters: {
          resolvedPaths
        },
        executor: async (params) => {
          const { resolvedPath } = params;

          // skip lazy compiled module and non-exist file
          if (
            VitePluginAdapter.isFarmInternalVirtualModule(resolvedPath) ||
            !existsSync(resolvedPath)
          ) {
            // for virtual modules that is not loaded by plugins, it should be treated as empty module
            // cause vite does not require load, vite can handle requests in middlewares
            if (!isAbsolute(resolvedPath)) {
              logger.info(
                `No plugins load virtual ${resolvedPath} in load hook. Farm load it as "export default await import('/@id/' + '${resolvedPath}');" by default for Vite Compatibility`
              );
              return {
                content: `export default await import('/@id/' + '${resolvedPath}');`,
                moduleType: 'js'
              };
            }

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
          resolvedPaths,
          moduleTypes: []
        },
        executor: async (params) => {
          const { content, moduleId, moduleType, resolvedPath } = params;

          // skip lazy compiled module and non-exist file
          if (VitePluginAdapter.isFarmInternalVirtualModule(resolvedPath)) {
            return null;
          }
          const cssModules = userConfig.compilation?.css?.modules?.paths ?? [
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
  jsPlugins: JsPlugin[],
  logger: Logger,
  mode: CompilationMode
) {
  const processPlugin = (plugin: any) => {
    const vitePluginAdapter = new VitePluginAdapter(
      plugin as any,
      userConfig,
      filters,
      logger,
      mode
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

function normalizeFilterPath(path: string): string {
  if (process.platform === 'win32') {
    return compatibleWin32Path(path);
  }

  return path;
}

function compatibleWin32Path(path: string): string {
  return path.replaceAll('/', '\\\\');
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

  if (plugin.renderResourcePot) {
    plugin.renderResourcePot.filters ??= {};

    if (
      !plugin.renderResourcePot?.filters?.moduleIds &&
      !plugin.renderResourcePot?.filters?.resourcePotTypes
    ) {
      throw new Error(
        `renderResourcePot hook of plugin ${plugin.name} must have at least one filter(like moduleIds or resourcePotTypes)`
      );
    }

    if (!plugin.renderResourcePot.filters?.resourcePotTypes) {
      plugin.renderResourcePot.filters.resourcePotTypes = [];
    } else if (!plugin.renderResourcePot.filters?.moduleIds) {
      plugin.renderResourcePot.filters.moduleIds = [];
    }
  }

  if (plugin.augmentResourceHash) {
    plugin.augmentResourceHash.filters ??= {};

    if (
      !plugin.augmentResourceHash?.filters?.moduleIds &&
      !plugin.augmentResourceHash?.filters?.resourcePotTypes
    ) {
      throw new Error(
        `augmentResourceHash hook of plugin ${plugin.name} must have at least one filter(like moduleIds or resourcePotTypes)`
      );
    }

    if (!plugin.augmentResourceHash.filters?.resourcePotTypes) {
      plugin.augmentResourceHash.filters.resourcePotTypes = [];
    } else if (!plugin.augmentResourceHash.filters?.moduleIds) {
      plugin.augmentResourceHash.filters.moduleIds = [];
    }
  }

  if (plugin.resolve?.filters?.importers?.length) {
    plugin.resolve.filters.importers =
      plugin.resolve.filters.importers.map(normalizeFilterPath);
  }

  if (plugin.load?.filters?.resolvedPaths?.length) {
    plugin.load.filters.resolvedPaths =
      plugin.load.filters.resolvedPaths.map(normalizeFilterPath);
  }

  if (plugin.transform?.filters?.resolvedPaths?.length) {
    plugin.transform.filters.resolvedPaths =
      plugin.transform.filters.resolvedPaths.map(normalizeFilterPath);
  }
  if (plugin.augmentResourceHash?.filters?.moduleIds) {
    plugin.augmentResourceHash.filters.moduleIds =
      plugin.augmentResourceHash.filters.moduleIds.map(normalizeFilterPath);
  }

  if (plugin.renderResourcePot?.filters?.moduleIds) {
    plugin.renderResourcePot.filters.moduleIds =
      plugin.renderResourcePot.filters.moduleIds.map(normalizeFilterPath);
  }
}
