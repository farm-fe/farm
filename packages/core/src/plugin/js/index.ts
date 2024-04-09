import {
  type JsPlugin,
  normalizeDevServerOptions,
  type UserConfig,
  Logger
} from '../../index.js';
import {
  DEFAULT_FILTERS,
  getCssModuleType,
  VITE_PLUGIN_DEFAULT_MODULE_TYPE
} from './utils.js';
import { VitePluginAdapter } from './vite-plugin-adapter.js';
import { existsSync, readFileSync } from 'node:fs';
import { resolveAsyncPlugins } from '../index.js';
import merge from '../../utils/merge.js';
import { CompilationMode } from '../../config/env.js';

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
  return process.platform === 'win32' ? compatibleWin32Path(path) : path;
}

function compatibleWin32Path(path: string): string {
  return path.replaceAll('/', '\\\\');
}

export function convertPlugin(plugin: JsPlugin): void {
  const ensureFilterExists = (filters: any, filterName: string): void => {
    filters[filterName] ??= [];
  };

  const ensureHookFiltersExist = (hook: any, hookName: string): void => {
    if (!hook) return;

    hook.filters ||= {};

    if (!hook.filters.moduleIds || !hook.filters.resourcePotTypes) {
      throw new Error(
        `${hookName} hook of plugin ${plugin.name} must have at least one filter(like moduleIds or resourcePotTypes)`
      );
    }

    ensureFilterExists(hook.filters, 'moduleIds');
    ensureFilterExists(hook.filters, 'resourcePotTypes');
  };

  if (plugin.transform) {
    if (
      !plugin.transform.filters?.moduleTypes &&
      !plugin.transform.filters?.resolvedPaths
    ) {
      throw new Error(
        `transform hook of plugin ${plugin.name} must have at least one filter(like moduleTypes or resolvedPaths)`
      );
    }

    ensureFilterExists(plugin.transform.filters, 'moduleTypes');
    ensureFilterExists(plugin.transform.filters, 'resolvedPaths');
  }

  const normalizeFilters = (filters: string[]): string[] => {
    return filters.map(normalizeFilterPath);
  };

  const hookNames = {
    renderResourcePot: 'renderResourcePot',
    augmentResourceHash: 'augmentResourceHash'
  };

  Object.keys(hookNames).forEach((hookKey) => {
    const hook = plugin[hookKey as keyof JsPlugin];
    ensureHookFiltersExist(hook, hookNames[hookKey as keyof typeof hookNames]);
  });

  if (plugin.resolve?.filters?.importers?.length) {
    plugin.resolve.filters.importers = normalizeFilters(
      plugin.resolve.filters.importers
    );
  }

  if (plugin.load?.filters?.resolvedPaths?.length) {
    plugin.load.filters.resolvedPaths = normalizeFilters(
      plugin.load.filters.resolvedPaths
    );
  }

  if (plugin.transform?.filters?.resolvedPaths?.length) {
    plugin.transform.filters.resolvedPaths = normalizeFilters(
      plugin.transform.filters.resolvedPaths
    );
  }

  if (plugin.augmentResourceHash?.filters?.moduleIds) {
    plugin.augmentResourceHash.filters.moduleIds = normalizeFilters(
      plugin.augmentResourceHash.filters.moduleIds
    );
  }

  if (plugin.renderResourcePot?.filters?.moduleIds) {
    plugin.renderResourcePot.filters.moduleIds = normalizeFilters(
      plugin.renderResourcePot.filters.moduleIds
    );
  }
}
