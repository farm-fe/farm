import { type TransformOptions, transformAsync } from '@babel/core';
import type { JsPlugin, ModuleType } from '@farmfe/core';

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

const defaultModuleTypes: ModuleType[] = ['js', 'jsx', 'ts', 'tsx'];
const defaultResolvedPaths: string[] = [];

export function babel(options: BabelOptions = { priority: 99 }): JsPlugin {
  return {
    name: options.name ?? 'js-plugin:babel',
    priority: options.priority,

    processModule: {
      filters: {
        moduleTypes: options.filters?.moduleTypes ?? defaultModuleTypes,
        resolvedPaths: options.filters?.resolvedPaths ?? defaultResolvedPaths
      },
      async executor(param) {
        const { content, moduleId } = param;

        const result = await transformAsync(content, {
          filename: moduleId,
          ...options.transformOptions
        });

        return {
          content: result?.code ?? ''
        };
      }
    }
  };
}
