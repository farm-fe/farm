import type { JsPlugin } from '@farmfe/core';
import type { ConfigPlugin, Config as SvgrOptions } from '@svgr/core';
import fs from 'fs';

export interface FarmSvgrPluginOptions {
  svgrOptions?: SvgrOptions;
  filters?: {
    resolvedPaths?: string[];
  };
}

export default function farmSvgrPlugin(
  options: FarmSvgrPluginOptions = {}
): JsPlugin {
  const { svgrOptions, filters } = options;

  return {
    name: '@farmfe/js-plugin-svgr',
    load: {
      filters: { resolvedPaths: filters?.resolvedPaths ?? ['\\.svg$'] },
      async executor(param) {
        if (
          param.query.some(
            ([key, _]) => key === 'raw' || key === 'url' || key === 'inline'
          )
        ) {
          return null;
        }

        const { transform } = await import('@svgr/core');
        const mod = await import('@svgr/plugin-jsx');
        const jsx = mod.default ?? mod;

        const svgCode = await fs.promises.readFile(param.resolvedPath, 'utf8');
        const componentCode = await transform(svgCode, svgrOptions, {
          filePath: param.resolvedPath,
          caller: {
            defaultPlugins: [jsx as unknown as ConfigPlugin]
          }
        });
        return {
          content: componentCode,
          moduleType: 'jsx'
        };
      }
    }
  };
}
