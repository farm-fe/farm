import { JsPlugin, UserConfig } from '@farmfe/core';
import type { StringOptions, CompileResult } from 'sass';
import {
  getAdditionContext,
  pluginName,
  throwError,
  tryRead
} from './options.js';
import { pathToFileURL } from 'url';
import { getSassImplementation } from './utils.js';
import path, { isAbsolute } from 'path';
import { existsSync } from 'fs';

export type SassPluginOptions = {
  sassOptions?: StringOptions<'async'>;
  filters?: {
    resolvedPaths?: string[];
    moduleTypes?: string[];
  };

  /**
   * - relative or absolute path
   * - globals file will be added to the top of the sass file
   * - when file changed, the file can't be hot-reloaded
   *
   * relative to project root or cwd
   */
  implementation?: string | undefined;
  globals?: string[];
  additionalData?:
    | string
    | ((content?: string, resolvePath?: string) => string | Promise<string>);
};

const DEFAULT_PATHS_REGEX = ['\\.(s[ac]ss)$'];

export default function farmSassPlugin(
  options: SassPluginOptions = {}
): JsPlugin {
  let farmConfig!: UserConfig['compilation'];
  const implementation = getSassImplementation(options.implementation);

  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore TODO fix it
  const cwd = () => farmConfig.root ?? process.cwd();

  const resolvedPaths = options.filters?.resolvedPaths ?? DEFAULT_PATHS_REGEX;

  return {
    name: pluginName,
    config: (config) => {
      farmConfig = config;
      return config;
    },
    load: {
      filters: { resolvedPaths },
      async executor(param) {
        if (param.query.length === 0 && existsSync(param.resolvedPath)) {
          const data = await tryRead(param.resolvedPath);
          return {
            content: data,
            moduleType: 'sass'
          };
        }

        return null;
      }
    },
    transform: {
      filters: {
        resolvedPaths: options.filters?.resolvedPaths,
        moduleTypes: options.filters?.moduleTypes ?? ['sass']
      },
      async executor(param, ctx) {
        try {
          const additionContext = await getAdditionContext(
            cwd(),
            options,
            param.resolvedPath,
            param.content,
            ctx
          );

          const sourceMapEnabled = ctx.sourceMapEnabled(param.moduleId);
          const sassImpl = await implementation;
          const { css, sourceMap } = (await sassImpl.compileStringAsync(
            `${additionContext}\n${param.content}`,
            {
              ...(options?.sassOptions ?? {}),
              sourceMap: options.sassOptions?.sourceMap ?? sourceMapEnabled,
              url: pathToFileURL(param.resolvedPath),
              importers: [
                {
                  async findFileUrl(url) {
                    if (!isAbsolute(url)) {
                      const relPath = path.join(
                        path.dirname(param.resolvedPath),
                        url
                      );

                      if (existsSync(relPath)) {
                        return pathToFileURL(relPath);
                      }
                    }
                    const result = await ctx.resolve(
                      {
                        source: url,
                        importer: param.moduleId,
                        kind: 'cssAtImport'
                      },
                      {
                        meta: {},
                        caller: '@farmfe/js-plugin-sass'
                      }
                    );

                    if (result?.resolvedPath) {
                      return pathToFileURL(result.resolvedPath);
                    }
                  }
                }
              ]
            } as StringOptions<'async'>
          )) as CompileResult;

          return {
            content: css,
            moduleType: 'css',
            sourceMap: sourceMap && JSON.stringify(sourceMap)
          };
        } catch (error) {
          throwError('transform', error);
        }

        return {
          content: '',
          moduleType: 'css'
        };
      }
    }
  };
}
