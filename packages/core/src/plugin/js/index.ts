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
    vitePlugin.forEach(processPlugin);
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

// export function convertPlugin(plugin: JsPlugin): void {
//   if (
//     plugin.transform &&
//     !plugin.transform.filters?.moduleTypes &&
//     !plugin.transform.filters?.resolvedPaths
//   ) {
//     throw new Error(
//       `transform hook of plugin ${plugin.name} must have at least one filter(like moduleTypes or resolvedPaths)`
//     );
//   }
//   if (plugin.transform) {
//     if (!plugin.transform.filters.moduleTypes) {
//       plugin.transform.filters.moduleTypes = [];
//     } else if (!plugin.transform.filters.resolvedPaths) {
//       plugin.transform.filters.resolvedPaths = [];
//     }
//   }

//   if (plugin.renderResourcePot) {
//     plugin.renderResourcePot.filters ??= {};

//     if (
//       !plugin.renderResourcePot?.filters?.moduleIds &&
//       !plugin.renderResourcePot?.filters?.resourcePotTypes
//     ) {
//       throw new Error(
//         `renderResourcePot hook of plugin ${plugin.name} must have at least one filter(like moduleIds or resourcePotTypes)`
//       );
//     }

//     if (!plugin.renderResourcePot.filters?.resourcePotTypes) {
//       plugin.renderResourcePot.filters.resourcePotTypes = [];
//     } else if (!plugin.renderResourcePot.filters?.moduleIds) {
//       plugin.renderResourcePot.filters.moduleIds = [];
//     }
//   }

//   if (plugin.augmentResourceHash) {
//     plugin.augmentResourceHash.filters ??= {};

//     if (
//       !plugin.augmentResourceHash?.filters?.moduleIds &&
//       !plugin.augmentResourceHash?.filters?.resourcePotTypes
//     ) {
//       throw new Error(
//         `augmentResourceHash hook of plugin ${plugin.name} must have at least one filter(like moduleIds or resourcePotTypes)`
//       );
//     }

//     if (!plugin.augmentResourceHash.filters?.resourcePotTypes) {
//       plugin.augmentResourceHash.filters.resourcePotTypes = [];
//     } else if (!plugin.augmentResourceHash.filters?.moduleIds) {
//       plugin.augmentResourceHash.filters.moduleIds = [];
//     }
//   }

//   if (plugin.resolve?.filters?.importers?.length) {
//     if (process.platform === 'win32') {
//       // replace / to \
//       plugin.resolve.filters.importers =
//         plugin.resolve.filters.importers.map(compatibleWin32Path);
//     }
//   }

//   if (plugin.load?.filters?.resolvedPaths?.length) {
//     if (process.platform === 'win32') {
//       // replace / to \
//       plugin.load.filters.resolvedPaths =
//         plugin.load.filters.resolvedPaths.map(compatibleWin32Path);
//     }
//   }

//   if (plugin.transform?.filters?.resolvedPaths?.length) {
//     if (process.platform === 'win32') {
//       // replace / to \
//       plugin.transform.filters.resolvedPaths =
//         plugin.transform.filters.resolvedPaths.map(compatibleWin32Path);
//     }
//   }
//   if (
//     plugin.augmentResourceHash?.filters?.moduleIds &&
//     process.platform === 'win32'
//   ) {
//     plugin.augmentResourceHash.filters.moduleIds =
//       plugin.augmentResourceHash.filters.moduleIds.map(compatibleWin32Path);
//   }

//   if (
//     plugin.renderResourcePot?.filters?.moduleIds &&
//     process.platform === 'win32'
//   ) {
//     plugin.renderResourcePot.filters.moduleIds =
//       plugin.renderResourcePot.filters.moduleIds.map(compatibleWin32Path);
//   }
// }

function ensureFilters(pluginPart: any, requiredFields: string[]): void {
  pluginPart.filters ??= {};

  const hasAtLeastOneFilter = requiredFields.some((field) =>
    Boolean(pluginPart.filters![field])
  );
  if (!hasAtLeastOneFilter) {
    throw new Error(
      `Farm Javascript Plugin part must have at least one filter: ${requiredFields.join(
        ' or '
      )}`
    );
  }

  requiredFields.forEach((field) => {
    pluginPart.filters![field] ??= [];
  });
}

function ensurePathCompatibility(pluginPart: any, fields: string[]): void {
  if (process.platform === 'win32' && pluginPart.filters) {
    fields.forEach((field) => {
      if (pluginPart.filters[field]?.length) {
        pluginPart.filters[field] =
          pluginPart.filters[field].map(compatibleWin32Path);
      }
    });
  }
}

export function convertPlugin(plugin: JsPlugin): void {
  if (plugin.transform) {
    ensureFilters(plugin.transform, ['moduleTypes', 'resolvedPaths']);
  }

  if (plugin.renderResourcePot) {
    ensureFilters(plugin.renderResourcePot, ['moduleIds', 'resourcePotTypes']);
  }

  if (plugin.augmentResourceHash) {
    ensureFilters(plugin.augmentResourceHash, [
      'moduleIds',
      'resourcePotTypes'
    ]);
  }

  const pathCompatibleParts = [
    { part: plugin.resolve, fields: ['importers'] },
    { part: plugin.load, fields: ['resolvedPaths'] },
    { part: plugin.transform, fields: ['resolvedPaths'] },
    { part: plugin.augmentResourceHash, fields: ['moduleIds'] },
    { part: plugin.renderResourcePot, fields: ['moduleIds'] }
  ];

  pathCompatibleParts.forEach(({ part, fields }) => {
    ensurePathCompatibility(part, fields);
  });
  
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
