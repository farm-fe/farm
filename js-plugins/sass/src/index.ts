import type {
  JsPlugin,
  UserConfig,
  PluginTransformHookParam,
  CompilationContext
} from '@farmfe/core';
import type { StringOptions, CompileResult, LegacyOptions } from 'sass';
import * as Sass from 'sass';
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

export type SassPluginOptions<Legacy = boolean> = {
  sassOptions?: Legacy extends false
    ? StringOptions<'async'>
    : LegacyOptions<'async'>;
  filters?: {
    resolvedPaths?: string[];
    moduleTypes?: string[];
  };
  /**
   * Use legacy sass API. E.g `sass.render` instead of `sass.compileStringAsync`.
   */
  legacy?: Legacy;
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
    config(config) {
      if (!config?.compilation?.resolve?.extensions) {
        config.compilation ??= {};
        config.compilation.resolve ??= {};
        config.compilation.resolve.extensions ??= [];
      }

      config.compilation.resolve.extensions = [
        ...new Set(config.compilation.resolve.extensions.concat('scss', 'sass'))
      ];
      return config;
    },
    configResolved: (config) => {
      farmConfig = config.compilation;
      const preprocessorOptions =
        config.compilation?.css?._viteCssOptions?.preprocessorOptions?.scss ??
        {};
      options.sassOptions = {
        ...options.sassOptions,
        ...preprocessorOptions
      };
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
          const compileCssParams = {
            transformParam: param,
            additionContext,
            sassImpl,
            sourceMapEnabled,
            options,
            ctx
          };
          const { css, sourceMap } = options.legacy
            ? await compileScssLegacy(compileCssParams)
            : await compileScss(compileCssParams);

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

interface CompileCssParams {
  transformParam: PluginTransformHookParam;
  additionContext: string;
  sassImpl: typeof Sass;
  sourceMapEnabled: boolean;
  options: SassPluginOptions;
  ctx: CompilationContext;
}

async function resolveDependency(
  url: string,
  transformParam: PluginTransformHookParam,
  ctx: CompilationContext
) {
  if (!isAbsolute(url)) {
    const relPath = path.join(path.dirname(transformParam.resolvedPath), url);

    if (existsSync(relPath)) {
      return relPath;
    }
  }

  const result = await ctx.resolve(
    {
      source: url,
      importer: transformParam.moduleId,
      kind: 'cssAtImport'
    },
    {
      meta: {},
      caller: '@farmfe/js-plugin-sass'
    }
  );

  if (result?.resolvedPath) {
    return result.resolvedPath;
  }
}

async function compileScss(param: CompileCssParams) {
  const {
    transformParam,
    additionContext,
    sassImpl,
    sourceMapEnabled,
    options,
    ctx
  } = param;
  const { css, sourceMap } = (await sassImpl.compileStringAsync(
    `${additionContext}\n${transformParam.content}`,
    {
      ...(options?.sassOptions ?? {}),
      sourceMap: options.sassOptions?.sourceMap ?? sourceMapEnabled,
      url: pathToFileURL(transformParam.resolvedPath),
      importers: [
        {
          async findFileUrl(url) {
            const resolvedPath = await resolveDependency(
              url,
              transformParam,
              ctx
            );

            if (resolvedPath) {
              return pathToFileURL(resolvedPath);
            }
          }
        }
      ]
    } as StringOptions<'async'>
  )) as CompileResult;

  return { css, sourceMap };
}

async function compileScssLegacy(param: CompileCssParams) {
  const {
    transformParam,
    additionContext,
    sassImpl,
    sourceMapEnabled,
    options,
    ctx
  } = param;

  return new Promise<{ css: string; sourceMap: unknown }>((resolve, reject) => {
    sassImpl.render(
      {
        ...(options?.sassOptions ?? {}),
        data: `${additionContext}\n${transformParam.content}`,
        sourceMap: options.sassOptions?.sourceMap ?? sourceMapEnabled,
        outFile: transformParam.resolvedPath,
        importer: [
          function (url, _prev, done) {
            resolveDependency(url, transformParam, ctx).then((resolvedPath) => {
              done({ file: resolvedPath });
            });
          }
        ]
      },
      (err, result) => {
        if (err) {
          reject(err);
          return;
        }

        resolve({ css: result.css.toString(), sourceMap: result.map });
      }
    );
  });
}
