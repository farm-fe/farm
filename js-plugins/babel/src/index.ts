import { transformAsync, type TransformOptions } from "@babel/core";
import type { ModuleType, JsPlugin } from "@farmfe/core";

export interface NormalizeFilterParams {
  moduleTypes: ModuleType[];
  resolvedPaths: string[];
}

export interface BabelOptions {
  filters?: Partial<NormalizeFilterParams>;
  transformOptions?: TransformOptions;
  /**
   * the priority of the plugin, large number means first to run
   * @default 99
   */
  priority: number;
  name?: string;
}

const defaultFilters: BabelOptions["filters"] = {
  moduleTypes: ["js", "jsx", "ts", "tsx"],
  resolvedPaths: [],
};

export function babel(options: BabelOptions = { priority: 99 }): JsPlugin {
  return {
    name: options.name ?? "js-plugin:babel",
    priority: options.priority,

    processModule: {
      filters: {
        moduleTypes: options.filters?.moduleTypes ?? defaultFilters.moduleTypes,
        resolvedPaths:
          options.filters?.resolvedPaths ?? defaultFilters.resolvedPaths,
      },
      async executor(param) {
        const { content, moduleId } = param;

        const result = await transformAsync(content, {
          filename: moduleId,
          ...options.transformOptions,
        });

        return {
          content: result.code,
        };
      },
    },
  };
}
