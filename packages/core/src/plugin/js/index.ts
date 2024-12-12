import { z } from 'zod';

import { fromZodError } from 'zod-validation-error';
import { CompilationMode } from '../../config/env.js';
import {
  type JsPlugin,
  type UserConfig,
  normalizeDevServerConfig
} from '../../index.js';
import merge from '../../utils/merge.js';
import { resolveAsyncPlugins } from '../index.js';

import { cssPluginUnwrap, cssPluginWrap } from './adapter-plugins/css.js';
import { defaultLoadPlugin } from './adapter-plugins/default-load.js';
import {
  PluginSchemaRegistry,
  createAugmentResourceHashSchema,
  createBuildEndSchema,
  createBuildStartSchema,
  createConfigResolvedSchema,
  createConfigSchema,
  createConfigureCompilerSchema,
  createConfigureServerSchema,
  createFinalizeResourcesSchema,
  createFinishSchema,
  createLoadSchema,
  createNameSchema,
  createPluginCacheLoadedSchema,
  createPrioritySchema,
  createRenderResourcePotSchema,
  createRenderStartSchema,
  createResolveSchema,
  createTransformHtmlSchema,
  createTransformSchema,
  createUpdateFinishedSchema,
  createUpdateModulesSchema,
  createWritePluginCacheSchema,
  createWriteResourcesSchema
} from './js-plugin-schema.js';

import { DEFAULT_FILTERS, normalizeFilterPath } from './utils.js';
import { VitePluginAdapter } from './vite-plugin-adapter.js';

export { VitePluginAdapter } from './vite-plugin-adapter.js';
export * from './js-plugin-schema.js';

type VitePluginType = object | (() => { vitePlugin: any; filters: string[] });
type VitePluginsType = VitePluginType[];

export async function handleVitePlugins(
  vitePlugins: VitePluginsType,
  userConfig: UserConfig,
  mode: CompilationMode
): Promise<JsPlugin[]> {
  const jsPlugins: JsPlugin[] = [];
  const filtersUnion = new Set<string>();

  if (vitePlugins.length) {
    userConfig = merge({}, userConfig, {
      compilation: userConfig.compilation,
      server: normalizeDevServerConfig(
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
    processVitePlugin(vitePlugin, userConfig, filters, jsPlugins, mode);
  }

  // if vitePlugins is not empty, append a load plugin to load file
  // this plugin is only for compatibility
  if (vitePlugins.length) {
    jsPlugins.push(
      defaultLoadPlugin({
        filtersUnion,
        userConfig
      })
    );
    jsPlugins.unshift(cssPluginWrap({ filtersUnion }));
    jsPlugins.push(cssPluginUnwrap({ filtersUnion }));
  }

  return jsPlugins;
}

export function processVitePlugin(
  vitePlugin: VitePluginType,
  userConfig: UserConfig,
  filters: string[],
  jsPlugins: JsPlugin[],
  mode: CompilationMode
) {
  const processPlugin = (plugin: any) => {
    const vitePluginAdapter = new VitePluginAdapter(
      plugin as any,
      userConfig,
      filters,
      mode
    );
    // @ts-ignore
    convertPluginVite(vitePluginAdapter);
    jsPlugins.push(vitePluginAdapter);
  };

  if (Array.isArray(vitePlugin)) {
    vitePlugin.forEach((plugin) => processPlugin(plugin));
  } else {
    processPlugin(vitePlugin);
  }
}

const schemaRegistry = new PluginSchemaRegistry();

schemaRegistry
  .register('name', createNameSchema)
  .register('priority', createPrioritySchema)
  .register('configureServer', createConfigureServerSchema)
  .register('configureCompiler', createConfigureCompilerSchema)
  .register('config', createConfigSchema)
  .register('configResolved', createConfigResolvedSchema)
  .register('buildStart', createBuildStartSchema)
  .register('resolve', createResolveSchema)
  .register('load', createLoadSchema)
  .register('transform', createTransformSchema)
  .register('buildEnd', createBuildEndSchema)
  .register('renderStart', createRenderStartSchema)
  .register('renderResourcePot', createRenderResourcePotSchema)
  .register('augmentResourceHash', createAugmentResourceHashSchema)
  .register('finalizeResources', createFinalizeResourcesSchema)
  .register('transformHtml', createTransformHtmlSchema)
  .register('writeResource', createWriteResourcesSchema)
  .register('pluginCacheLoaded', createPluginCacheLoadedSchema)
  .register('writePluginCache', createWritePluginCacheSchema)
  .register('finish', createFinishSchema)
  .register('updateFinished', createUpdateFinishedSchema)
  .register('updateModules', createUpdateModulesSchema);

export function convertPlugin(plugin: JsPlugin) {
  try {
    const pluginSchema = schemaRegistry.createPluginSchema(plugin?.name);
    return pluginSchema.parse(plugin);
  } catch (err) {
    const validationError = fromZodError(err, {
      prefix: 'Failed to verify js plugin schema'
    });
    const pluginName = plugin?.name || 'undefined';
    throw new Error(
      `${validationError.toString()}. \n Please check '${pluginName}' plugin passes these attributes correctly.`
    );
  }
}

export function convertPluginVite(plugin: JsPlugin): void {
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
