import { JsPlugin, UserConfig } from '@farmfe/core';
import fs from 'fs';
import sass, { StringOptions } from 'sass';
import { URL } from 'node:url';
import { getAdditionContext } from './options';

export interface SassOptions {
  sassOption?: StringOptions<'async'>;
  /**
   * - relative or absolute path
   * - globals file will be added to the top of the sass file
   * - when file changed, the file can't be hot-reloaded
   *
   * relative to project root or cwd
   */
  globals?: string[];
  content?: string;
  sourceMap?: boolean;
}

const pluginName = 'farm-sass-plugin';

function throwError(type: string, error: Error) {
  console.error(`[${pluginName} ${type} Error] ${error}`);
}

async function tryRead(filename: string) {
  try {
    return fs.promises.readFile(filename, 'utf-8');
  } catch (e) {
    throwError('read', e);
  }
}

export default function farmSassPlugin(options: SassOptions = {}): JsPlugin {
  let farmConfig!: UserConfig;
  let cacheAdditionContext: string | null;

  const cwd = () => farmConfig.root ?? process.cwd();

  return {
    name: pluginName,
    config: (param, context, hookContext) => (farmConfig = param),
    load: {
      filters: { resolvedPaths: ['.scss$'] },
      async executor(param, context, hookContext) {
        const data = await tryRead(param.resolvedPath);
        return {
          content: data,
          moduleType: 'css',
        };
      },
    },

    transform: {
      filters: {
        resolvedPaths: ['.scss$'],
      },
      async executor(param, context, hookContext) {
        try {
          const additionContext =
            cacheAdditionContext ??
            (cacheAdditionContext = getAdditionContext(cwd(), options));

          const { css, sourceMap } = await sass.compileStringAsync(
            `${additionContext}\n${param.content}`,
            {
              sourceMap: Boolean(
                options.sourceMap ?? farmConfig?.compilation?.sourcemap
              ),
              url: new URL(`file://${param.resolvedPath}}`),
            }
          );

          return {
            content: css,
            moduleType: 'css',
            sourceMap: sourceMap && JSON.stringify(sourceMap),
          };
        } catch (error) {
          console.error(error);
        }
        return {
          content: '',
          moduleType: 'css',
        };
      },
    },
  };
}
