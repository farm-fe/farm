import { JsPlugin, UserConfig } from '@farmfe/core';
import sass, { StringOptions } from 'sass';
import { getAdditionContext, pluginName, tryRead } from './options.js';
import { pathToFileURL } from 'url';

export interface SassOptions {
  match?: string[];
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

const defaultMatch = ['\\.scss$'];

export default function farmSassPlugin(options: SassOptions = {}): JsPlugin {
  let farmConfig!: UserConfig;
  let cacheAdditionContext: string | null;

  const cwd = () => farmConfig.root ?? process.cwd();

  const match = (options.match ?? defaultMatch).map((item) => item.toString());

  return {
    name: pluginName,
    config: (param) => (farmConfig = param),
    load: {
      filters: { resolvedPaths: match },
      async executor(param) {
        const data = await tryRead(param.resolvedPath);
        return {
          content: data,
          moduleType: 'css',
        };
      },
    },

    transform: {
      filters: {
        resolvedPaths: match,
      },
      async executor(param) {
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
              url: pathToFileURL(param.resolvedPath),
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
