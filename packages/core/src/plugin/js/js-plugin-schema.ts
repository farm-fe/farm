// ✨ farm js plugin schema all in zod
import { z } from 'zod';

import { DEFAULT_FILTERS, normalizeFilterPath } from './utils.js';

const transformFilterSchema = z.object({
  moduleTypes: z.array(z.string()).optional().default([]),
  resolvedPaths: z.array(z.string()).optional().default([])
});

const transformSchema = z
  .object({
    filters: transformFilterSchema
      .refine(
        (data) => {
          return data.moduleTypes.length > 0 || data.resolvedPaths.length > 0;
        },
        {
          message:
            'transform hook must have at least one filter (moduleTypes or resolvedPaths)'
        }
      )
      .default({
        moduleTypes: [],
        resolvedPaths: []
      })
  })
  .transform((transform) => {
    const { filters } = transform;
    if (filters.resolvedPaths && filters.resolvedPaths.length > 0) {
      filters.resolvedPaths = filters.resolvedPaths.map(normalizeFilterPath);
    }
    return { ...transform, filters };
  });

export class PluginSchemaRegistry {
  private schemas: Map<string, (pluginName: string) => z.ZodSchema> = new Map();

  register(
    hookName: string,
    schemaFactory: (pluginName: string) => z.ZodSchema
  ) {
    this.schemas.set(hookName, schemaFactory);
    return this;
  }

  createPluginSchema(pluginName: string) {
    const schemaShape = Object.fromEntries(
      Array.from(this.schemas.entries()).map(([hookName, factory]) => [
        hookName,
        factory(pluginName).optional()
      ])
    );

    return z.object({
      name: z.string(),
      ...schemaShape
    });
  }
}
