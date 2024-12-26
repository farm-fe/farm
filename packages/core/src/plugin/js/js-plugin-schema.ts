// ✨ farm js plugin schema all in zod
import { z } from 'zod';

import { normalizeFilterPath } from './utils.js';

const EmptyRecordSchema = z.any();

const CallbackSchema = z
  .function()
  .args(EmptyRecordSchema)
  .returns(z.union([z.void(), z.promise(z.void())]));

const CallbackSchemaNotArgs = z
  .function()
  .returns(z.union([z.void(), z.promise(z.void())]));

const updateModulesCallbackSchema = z
  .function()
  .returns(
    z.union([
      z.array(z.any()),
      z.promise(z.array(z.any())),
      z.void(),
      z.promise(z.void())
    ])
  )
  .optional();

// name schema
export const nameSchema = z.string().min(1);

// priority schema
export const prioritySchema = z.number().int().default(100).optional();

// config schema
export const configSchema = z
  .function()
  .args(z.record(z.string(), z.any()))
  .returns(
    z.union([
      z.record(z.string(), z.any()),
      z.promise(z.record(z.string(), z.any()))
    ])
  )
  .optional();

// configResolved schema
export const configResolvedSchema = z
  .function()
  .args(z.record(z.string(), z.any()))
  .returns(z.union([z.void(), z.promise(z.void())]))
  .optional();

export const configureServerSchema = z
  .function()
  .args(z.any())
  .returns(z.union([z.void(), z.promise(z.void())]))
  .optional();

export const configureCompilerSchema = z
  .function()
  .args(z.any())
  .returns(z.union([z.void(), z.promise(z.void())]))
  .optional();

export const loadFilterSchema = z
  .object({
    resolvedPaths: z.array(z.string()).optional().default([])
  })
  .transform((data) => ({
    resolvedPaths: data.resolvedPaths ?? []
  }));

export const resolveFilterSchema = z
  .object({
    importers: z.array(z.string()).optional().default([]),
    sources: z.array(z.string()).optional().default([])
  })
  .transform((data) => ({
    importers: data.importers ?? [],
    sources: data.sources ?? []
  }));

export const transformFilterSchema = z
  .object({
    moduleTypes: z.array(z.string()).optional().default([]),
    resolvedPaths: z.array(z.string()).optional().default([])
  })
  .transform((data) => ({
    moduleTypes: data.moduleTypes ?? [],
    resolvedPaths: data.resolvedPaths ?? []
  }));

export const renderResourcePotSchema = z
  .object({
    resourcePotTypes: z.array(z.string()).optional().default([]),
    moduleIds: z.array(z.string()).optional().default([])
  })
  .transform((data) => ({
    resourcePotTypes: data.resourcePotTypes ?? [],
    moduleIds: data.moduleIds ?? []
  }));

export const augmentResourceHashSchema = z
  .object({
    resourcePotTypes: z.array(z.string()).optional().default([]),
    moduleIds: z.array(z.string()).optional().default([])
  })
  .transform((data) => ({
    resourcePotTypes: data.resourcePotTypes ?? [],
    moduleIds: data.moduleIds ?? []
  }));

export const createNameSchema = (name: string) => {
  return z
    .string()
    .min(1)
    .refine(
      (data) => {
        return !!data;
      },
      {
        message: `\n 'name' of plugin '${name}' is required`
      }
    );
};

export const createPrioritySchema = (name: string) => {
  return prioritySchema.refine(
    () => {
      return true;
    },
    {
      message: `\n 'priority' of plugin '${name}' must be greater type of number`
    }
  );
};

export const createConfigSchema = (name: string) => {
  return configSchema.superRefine((data, ctx) => {
    if (typeof data !== 'function') {
      ctx.addIssue({
        code: z.ZodIssueCode.invalid_type,
        expected: 'function',
        received: typeof data,
        message: `\n plugin '${name}' config hook must be a function:
        - Function signature: (config: UserConfig) => UserConfig | Promise<UserConfig>
        - Purpose: Modify or extend configuration
        - Parameter: Receives current user configuration
        - Returns: Modified configuration or its Promise`
      });
    }
  });
};

export const createConfigResolvedSchema = (name: string) => {
  return configResolvedSchema.superRefine((data, ctx) => {
    if (typeof data !== 'function') {
      ctx.addIssue({
        code: z.ZodIssueCode.invalid_type,
        expected: 'function',
        received: typeof data,
        message: `\n plugin '${name}' configResolved hook:
        - Function signature: (config: ResolvedUserConfig) => void | Promise<void>
        - Purpose: Handle the resolved configuration
        - Parameter: Final resolved configuration object
        - Returns: void or Promise<void>
        - Note: This hook is called after all config hooks have been applied`
      });
    }
  });
};

export const createConfigureServerSchema = (name: string) => {
  return configureServerSchema.superRefine((data, ctx) => {
    if (typeof data !== 'function') {
      ctx.addIssue({
        code: z.ZodIssueCode.invalid_type,
        expected: 'function',
        received: typeof data,
        message: `\n plugin '${name}' configureServer hook:
        - Function signature: (server: Server) => void | Promise<void>
        - Purpose: Configure the server
        - Parameter: Server instance
        - Returns: void or Promise<void>`
      });
    }
  });
};

export const createConfigureCompilerSchema = (name: string) => {
  return configureCompilerSchema.superRefine((data, ctx) => {
    if (typeof data !== 'function') {
      ctx.addIssue({
        code: z.ZodIssueCode.invalid_type,
        expected: 'function',
        received: typeof data,
        message: `\n plugin '${name}' configureCompiler hook:
        - Function signature: (compiler: Compiler) => void | Promise<void>
        - Purpose: Configure the compiler
        - Parameter: Compiler instance
        - Returns: void or Promise<void>`
      });
    }
  });
};

export const createBuildStartSchema = (name: string) => {
  return z
    .object({
      executor: CallbackSchema
    })
    .refine(
      (data) => {
        return !!data.executor;
      },
      {
        message: `\n 'buildStart' hook of plugin '${name}' must have an executor function`
      }
    )
    .optional();
};

export const createBuildEndSchema = (name: string) => {
  return z
    .object({
      executor: CallbackSchema
    })
    .refine(
      (data) => {
        return !!data.executor;
      },
      {
        message: `\n 'buildEnd' hook of plugin '${name}' must have an executor function`
      }
    )
    .optional();
};

export const createLoadSchema = (name: string) => {
  return z.object({
    filters: loadFilterSchema
      .refine(
        (data) => {
          return data.resolvedPaths.length > 0;
        },
        {
          message: `\n 'load' hook of plugin '${name}' must have at least one filter(like resolvedPaths)`
        }
      )
      .default({
        resolvedPaths: []
      }),
    executor: z.function()
  });
};

export const createResolveSchema = (name: string) => {
  return z.object({
    filters: resolveFilterSchema
      .refine(
        (data) => {
          return data.importers.length > 0 || data.sources.length > 0;
        },
        {
          message: `\n 'resolve' hook of plugin '${name}' must have at least one filter(like importers or sources)`
        }
      )
      .default({
        importers: [],
        sources: []
      }),
    executor: z.function()
  });
};

export const createTransformSchema = (name: string) => {
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

export const createRenderStartSchema = (name: string) => {
  return z
    .object({
      executor: CallbackSchema
    })
    .refine(
      (data) => {
        return !!data.executor;
      },
      {
        message: `\n 'renderStart' hook of plugin '${name}' must have an executor function`
      }
    )
    .optional();
};

export const createRenderResourcePotSchema = (name: string) => {
  return z
    .object({
      filters: renderResourcePotSchema
        .refine(
          (data) => {
            return (
              data.resourcePotTypes.length > 0 || data.moduleIds.length > 0
            );
          },
          {
            message: `\n 'renderResourcePot' hook of plugin '${name}' must have at least one filter(like moduleIds or resourcePotTypes)`
          }
        )
        .default({
          resourcePotTypes: [],
          moduleIds: []
        }),
      executor: z.function()
    })
    .transform((renderResourcePot) => {
      const { filters } = renderResourcePot;
      if (filters.moduleIds && filters.moduleIds.length > 0) {
        filters.moduleIds = filters.moduleIds.map(normalizeFilterPath);
      }
      return { ...renderResourcePot, filters };
    });
};

export const createAugmentResourceHashSchema = (name: string) => {
  return z.object({
    filters: augmentResourceHashSchema
      .refine(
        (data) => {
          return data.resourcePotTypes.length > 0 || data.moduleIds.length > 0;
        },
        {
          message: `\n 'augmentResourceHash' hook of plugin '${name}' must have at least one filter(like moduleIds or resourcePotTypes)`
        }
      )
      .default({
        resourcePotTypes: [],
        moduleIds: []
      }),
    executor: z.function()
  });
};

export const createFinalizeResourcesSchema = (name: string) => {
  return z
    .object({
      executor: z.function()
    })
    .refine(
      (data) => {
        return !!data.executor;
      },
      {
        message: `\n 'finalizeResources' hook of plugin '${name}' must have an executor function`
      }
    )
    .optional();
};

export const createTransformHtmlSchema = (name: string) => {
  return z
    .object({
      order: z.number().int().optional(),
      executor: CallbackSchema
    })
    .refine(
      (data) => {
        return !!data.executor;
      },
      {
        message: `\n 'transformHtml' hook of plugin '${name}' must have an executor function`
      }
    )
    .optional();
};

export const createWriteResourcesSchema = (name: string) => {
  return z
    .object({
      executor: z.function()
    })
    .refine(
      (data) => {
        return !!data.executor;
      },
      {
        message: `\n 'writeResources' hook of plugin '${name}' must have an executor function`
      }
    )
    .optional();
};

export const createPluginCacheLoadedSchema = (name: string) => {
  return z
    .object({
      executor: CallbackSchema
    })
    .refine(
      (data) => {
        return !!data.executor;
      },
      {
        message: `\n 'pluginCacheLoaded' hook of plugin '${name}' must have an executor function`
      }
    )
    .optional();
};

export const createWritePluginCacheSchema = (name: string) => {
  return z
    .object({
      executor: CallbackSchema
    })
    .refine(
      (data) => {
        return !!data.executor;
      },
      {
        message: `\n 'writePluginCache' hook of plugin '${name}' must have an executor function`
      }
    )
    .optional();
};

export const createFinishSchema = (name: string) => {
  return z
    .object({
      executor: CallbackSchemaNotArgs
    })
    .refine(
      (data) => {
        return !!data.executor;
      },
      {
        message: `\n 'finish' hook of plugin '${name}' must have an executor function`
      }
    )
    .optional();
};

export const createUpdateFinishedSchema = (name: string) => {
  return z
    .object({
      executor: CallbackSchemaNotArgs
    })
    .refine(
      (data) => {
        return !!data.executor;
      },
      {
        message: `\n 'updateFinished' hook of plugin '${name}' must have an executor function`
      }
    )
    .optional();
};

export const createUpdateModulesSchema = (name: string) => {
  return z
    .object({
      executor: updateModulesCallbackSchema
    })
    .refine(
      (data) => {
        return !!data.executor;
      },
      {
        message: `\n 'updateModules' hook of plugin '${name}' must have an executor function`
      }
    )
    .optional();
};

export class PluginSchemaRegistry {
  private schemas: Map<string, (pluginName: string) => z.ZodSchema> = new Map();

  register(
    hookName: string,
    schemaFactory: (pluginName: string) => z.ZodSchema
  ) {
    this.schemas.set(hookName, schemaFactory);
    return this;
  }

  createPluginSchema(pluginName: string | undefined) {
    const schemaShape = Object.fromEntries(
      Array.from(this.schemas.entries()).map(([hookName, factory]) => [
        hookName,
        hookName === 'name'
          ? factory(pluginName)
          : factory(pluginName).optional()
      ])
    );
    return z
      .object({
        ...schemaShape
      })
      .strict();
  }
}
