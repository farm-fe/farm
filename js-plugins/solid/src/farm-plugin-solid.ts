import { type JsPlugin, type UserConfig } from '@farmfe/core';
import { type TransformOptions, transformAsync } from '@babel/core';
import babelPresetTs from '@babel/preset-typescript';
import babelPresetSolid from 'babel-preset-solid';
// eslint-disable-next-line
// @ts-ignore
// change to solid-js/web when resolve support conditional exports
// TODO: HMR support with solid-refresh
// import solidRefresh from 'solid-refresh/dist/babel';
import { tryReadFile } from './utils';

export interface PluginOptions {
  ssr?: boolean;
  solid?: {};
  typescript?: {};
}

const dynamicImportPrefix = 'virtual:FARMFE_DYNAMIC_IMPORT:';

export default function solid(options: PluginOptions = {}): JsPlugin {
  let farmConfig: UserConfig['compilation'];

  return {
    name: 'farm-plugin-solid',
    config(config) {
      return (farmConfig = config ?? {});
    },
    load: {
      filters: {
        resolvedPaths: ['.(j|t)sx?$'],
      },
      async executor(params, ctx) {
        const { resolvedPath } = params;
        const content = await tryReadFile(resolvedPath);
        return {
          content,
          moduleType: 'js',
        };
      },
    },
    transform: {
      filters: {
        resolvedPaths: ['.(j|t)sx?$'],
      },
      async executor(params, ctx) {
        const { resolvedPath } = params;
        let path = resolvedPath;

        // TBD: replace virtual module prefix to avoid error when reading file
        if (resolvedPath.startsWith(dynamicImportPrefix)) {
          path = path.replace(dynamicImportPrefix, '');
        }

        const content = await tryReadFile(path);
        const solidOptions: { generate: 'ssr' | 'dom'; hydratable: boolean } = {
          generate: 'dom',
          hydratable: false,
          ...(options.solid ?? {}),
        };

        if (options.ssr) {
          solidOptions.hydratable = true;
        }

        const babelOptions: TransformOptions = {
          babelrc: false,
          configFile: false,
          root: farmConfig.root,
          filename: path,
          sourceFileName: path,
          presets: [[babelPresetSolid, solidOptions]],
          // HACK: should be removed when resolve support conditional exports
          plugins: [
            {
              visitor: {
                ImportDeclaration(path) {
                  if (path.node.source.value === 'solid-js/web') {
                    path.node.source.value = 'solid-js/web/dist/web';
                  }
                },
              },
            },
          ],
          // sourceMaps: true
        };

        const tsOptions = options.typescript ?? {};
        const shouldBeProcessedWithTypescript =
          /\.tsx?$/.test(path) || options.typescript;

        if (shouldBeProcessedWithTypescript) {
          babelOptions.presets.push([babelPresetTs, tsOptions]);
        }

        const { code, map } = await transformAsync(content, babelOptions);
        return {
          content: code,
          moduleType: 'js',
          // TODO: generate sourceMap with configuration
          // sourceMap: JSON.stringify(map),
        };
      },
    },
  };
}
