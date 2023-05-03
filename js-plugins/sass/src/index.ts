import { JsPlugin, UserConfig } from '@farmfe/core';
import { StringOptions } from 'sass';
import { getAdditionContext, pluginName, tryRead } from './options.js';
import { pathToFileURL } from 'url';
import { getSassImplementation } from './utils.js';

export type SassPluginOptions = StringOptions<'sync'> & {
  match?: string[];
  /**
   * - relative or absolute path
   * - globals file will be added to the top of the sass file
   * - when file changed, the file can't be hot-reloaded
   *
   * relative to project root or cwd
   */
  implementation?: string;
  globals?: string[];
  content?: string;
  sourceMap?: boolean;
};

const defaultMatch = ['\\.(s[ac]ss)$'];

export default function farmSassPlugin(
  options: SassPluginOptions = {}
): JsPlugin {
  let farmConfig!: UserConfig;
  let cacheAdditionContext: string | null;
  const implementation = getSassImplementation(options.implementation);
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
          moduleType: 'sass'
        };
      }
    },
    transform: {
      filters: {
        resolvedPaths: match
      },
      async executor(param) {
        try {
          const additionContext =
            cacheAdditionContext ??
            (cacheAdditionContext = getAdditionContext(cwd(), options));

          const { css, sourceMap } = await (
            await implementation
          ).compileStringAsync(`${additionContext}\n${param.content}`, {
            sourceMap: Boolean(
              options.sourceMap ?? farmConfig?.compilation?.sourcemap
            ),
            url: pathToFileURL(param.resolvedPath)
          });

          return {
            content: css,
            moduleType: 'css',
            sourceMap: sourceMap && JSON.stringify(sourceMap)
          };
        } catch (error) {
          console.error(error);
        }

        return {
          content: '',
          moduleType: 'css'
        };
      }
    }
  };
}
