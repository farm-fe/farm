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
import { PluginSchemaRegistry } from './js-plugin-schema.js';
import { DEFAULT_FILTERS, normalizeFilterPath } from './utils.js';
import { VitePluginAdapter } from './vite-plugin-adapter.js';

// export * from './jsPluginAdapter.js';
export { VitePluginAdapter } from './vite-plugin-adapter.js';

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

// export function convertPlugin(plugin: JsPlugin): void {
//   console.log(plugin);

//   if (
//     plugin.transform &&
//     !plugin.transform.filters?.moduleTypes &&
//     !plugin.transform.filters?.resolvedPaths
//   ) {
//     throw Error(
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
//     plugin.resolve.filters.importers =
//       plugin.resolve.filters.importers.map(normalizeFilterPath);
//   }

//   if (plugin.load?.filters?.resolvedPaths?.length) {
//     plugin.load.filters.resolvedPaths =
//       plugin.load.filters.resolvedPaths.map(normalizeFilterPath);
//   }

//   if (plugin.transform?.filters?.resolvedPaths?.length) {
//     plugin.transform.filters.resolvedPaths =
//       plugin.transform.filters.resolvedPaths.map(normalizeFilterPath);
//   }
//   if (plugin.augmentResourceHash?.filters?.moduleIds) {
//     plugin.augmentResourceHash.filters.moduleIds =
//       plugin.augmentResourceHash.filters.moduleIds.map(normalizeFilterPath);
//   }

//   if (plugin.renderResourcePot?.filters?.moduleIds) {
//     plugin.renderResourcePot.filters.moduleIds =
//       plugin.renderResourcePot.filters.moduleIds.map(normalizeFilterPath);
//   }
// }

const transformFilterSchema = z.object({
  moduleTypes: z.array(z.string()).optional().default([]),
  resolvedPaths: z.array(z.string()).optional().default([])
});

const createTransformSchema = (name: string) => {
  return z
    .object({
      filters: transformFilterSchema
        .refine(
          (data) => {
            return data.moduleTypes.length > 0 || data.resolvedPaths.length > 0;
          },
          {
            message: `\n transform hook of plugin '${name}' must have at least one filter(like moduleTypes or resolvedPaths)`
          }
        )
        .default({
          moduleTypes: [],
          resolvedPaths: []
        }),
      executor: z.function()
    })
    .transform((transform) => {
      const { filters } = transform;
      if (filters.resolvedPaths && filters.resolvedPaths.length > 0) {
        filters.resolvedPaths = filters.resolvedPaths.map(normalizeFilterPath);
      }
      return { ...transform, filters };
    });
};

const filterSchema = z.object({
  moduleTypes: z.array(z.string()).optional().default([]),
  resolvedPaths: z.array(z.string()).optional().default([]),
  moduleIds: z.array(z.string()).optional().default([]),
  resourcePotTypes: z.array(z.string()).optional().default([]),
  importers: z
    .array(z.string())
    .optional()
    .default([])
    .transform((arr) => arr.map(normalizeFilterPath))
});

// const transformSchema = z.object({
//   filters: filterSchema
//     .refine(
//       (data) => {
//         return data.moduleTypes.length > 0 || data.resolvedPaths.length > 0
//       },
//       {
//         message:
//           'transform hook must have at least one filter (moduleTypes or resolvedPaths)',
//       }
//     )
//     .default({
//       moduleTypes: [],
//       resolvedPaths: [],
//     }),
// }).transform((transform) => {
//   if (transform.filters.resolvedPaths.length > 0) {
//     transform.filters.resolvedPaths = transform.filters.resolvedPaths.map(normalizeFilterPath);
//   }
//   return transform;
// });

const renderResourcePotSchema = z
  .object({
    filters: filterSchema
      .refine(
        (data) => data.moduleIds.length > 0 || data.resourcePotTypes.length > 0,
        {
          message:
            'renderResourcePot hook must have at least one filter (moduleIds or resourcePotTypes)'
        }
      )
      .default({})
  })
  .transform((renderResourcePot) => {
    if (renderResourcePot.filters.moduleIds.length > 0) {
      renderResourcePot.filters.moduleIds =
        renderResourcePot.filters.moduleIds.map(normalizeFilterPath);
    }
    return renderResourcePot;
  });

const augmentResourceHashSchema = z
  .object({
    filters: filterSchema
      .refine(
        (data) => data.moduleIds.length > 0 || data.resourcePotTypes.length > 0,
        {
          message:
            'augmentResourceHash hook must have at least one filter (moduleIds or resourcePotTypes)'
        }
      )
      .default({})
  })
  .transform((augmentResourceHash) => {
    if (augmentResourceHash.filters.moduleIds.length > 0) {
      augmentResourceHash.filters.moduleIds =
        augmentResourceHash.filters.moduleIds.map(normalizeFilterPath);
    }
    return augmentResourceHash;
  });

const resolveSchema = z
  .object({
    filters: z
      .object({
        importers: z
          .array(z.string())
          .optional()
          .transform((arr) => arr.map(normalizeFilterPath))
      })
      .optional()
  })
  .transform((resolve) => {
    if (resolve.filters?.importers?.length) {
      resolve.filters.importers =
        resolve.filters.importers.map(normalizeFilterPath);
    }
    return resolve;
  });

const loadSchema = z
  .object({
    filters: z
      .object({
        resolvedPaths: z
          .array(z.string())
          .optional()
          .transform((arr) => arr.map(normalizeFilterPath))
      })
      .optional()
  })
  .transform((load) => {
    if (load.filters?.resolvedPaths?.length) {
      load.filters.resolvedPaths =
        load.filters.resolvedPaths.map(normalizeFilterPath);
    }
    return load;
  });

const schemaRegistry = new PluginSchemaRegistry();

schemaRegistry.register('transform', createTransformSchema);
// .register('renderResourcePot', createRenderResourcePotSchema)
// .register('augmentResourceHash', createAugmentResourceHashSchema)
// .register('resolve', createResolveSchema)
// .register('load', createLoadSchema);

export function convertPlugin(plugin: JsPlugin) {
  const { name } = z.object({ name: z.string() }).parse(plugin);
  // const pluginSchema = z.object({
  //   name: z.string(),
  //   transform: createTransformSchema(name).optional(),
  //   renderResourcePot: renderResourcePotSchema.optional(),
  //   augmentResourceHash: augmentResourceHashSchema.optional(),
  //   resolve: resolveSchema.optional(),
  //   load: loadSchema.optional(),
  // });
  try {
    const pluginSchema = schemaRegistry.createPluginSchema(name);
    const res = pluginSchema.parse(plugin);
    return res;
  } catch (err) {
    const validationError = fromZodError(err, {
      prefix: 'Failed to verify js plugin schema'
    });
    throw new Error(
      `${validationError.toString()}. \n Please check '${name}' plugin passes these attributes correctly.`
    );
  }
}
