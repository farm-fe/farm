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
  createWritePluginCacheSchema
} from './js-plugin-schema.js';
import { DEFAULT_FILTERS } from './utils.js';
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
    let vitePluginAdapter = new VitePluginAdapter(
      plugin as any,
      userConfig,
      filters,
      mode
    );
    // @ts-ignore
    vitePluginAdapter = convertPlugin(vitePluginAdapter);
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
  .register('writeResource', createFinalizeResourcesSchema)
  .register('pluginCacheLoaded', createPluginCacheLoadedSchema)
  .register('writePluginCache', createWritePluginCacheSchema)
  .register('finish', createFinishSchema)
  .register('updateFinished', createUpdateFinishedSchema)
  .register('updateModules', createUpdateModulesSchema);

export function convertPlugin(plugin: JsPlugin) {
  try {
    const pluginSchema = schemaRegistry.createPluginSchema(plugin?.name);

    const res = pluginSchema.parse(plugin);

    return res;
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
