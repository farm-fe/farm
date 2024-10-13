import { existsSync, readFileSync } from 'node:fs';
import { isAbsolute } from 'node:path';
import { UserConfig } from '../../../config/types.js';
import { Logger } from '../../../utils/logger.js';
import { JsPlugin } from '../../type.js';
import {
  VITE_PLUGIN_DEFAULT_MODULE_TYPE,
  getCssModuleType,
  normalizeFilterPath
} from '../utils.js';
import { VitePluginAdapter } from '../vite-plugin-adapter.js';

export function defaultLoadPlugin(options: {
  filtersUnion: Set<string>;
  userConfig: UserConfig;
}): JsPlugin {
  const logger = new Logger();
  const { filtersUnion, userConfig } = options;
  const resolvedPaths = Array.from(filtersUnion).map(normalizeFilterPath);

  return {
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
  };
}
