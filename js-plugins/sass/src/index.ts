import { existsSync, statSync } from 'fs';
import path, { isAbsolute } from 'path';
import { fileURLToPath, pathToFileURL } from 'url';
import type {
  CompilationContext,
  JsPlugin,
  PluginTransformHookParam,
  UserConfig
} from '@farmfe/core';
import { getAdditionContext, rebaseUrls } from '@farmfe/core';
import { readFile } from 'fs/promises';
import type { CompileResult, LegacyOptions, StringOptions } from 'sass';
import * as Sass from 'sass';
import { pluginName, throwError, tryRead } from './options.js';
import { getSassImplementation } from './utils.js';

export type SassPluginOptions<Legacy = boolean> = {
  sassOptions?: Partial<
    Legacy extends false ? StringOptions<'async'> : LegacyOptions<'async'>
  >;
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

  // @ts-ignore TODO fix it
  const cwd = () => farmConfig.root ?? process.cwd();

  const resolvedPaths = options.filters?.resolvedPaths ?? DEFAULT_PATHS_REGEX;
  // enable legacy mode by default
  options.legacy = options.legacy ?? true;

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
            ctx,
            pluginName
          );

          const sourceMapEnabled = ctx.sourceMapEnabled(param.moduleId);
          const sassImpl = await implementation;
          const compileCssParams = {
            transformParam: param,
            additionContext,
            sassImpl,
            sourceMapEnabled,
            options,
            ctx,
            root: cwd()
          };
          const { css, sourceMap } = options.legacy
            ? await compileScssLegacy(compileCssParams)
            : await compileScss(compileCssParams);

          return {
            content: css,
            moduleType: 'css',
            sourceMap:
              typeof sourceMap === 'object'
                ? JSON.stringify(sourceMap)
                : (sourceMap as string | undefined)
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
  root: string;
}

async function resolveDependencyWithPrefix(
  url: string,
  transformParam: PluginTransformHookParam,
  prefix: string,
  ctx: CompilationContext
) {
  const filename = path.posix.join(
    path.posix.dirname(url),
    `${prefix}${path.posix.basename(url)}`
  );

  const result = await ctx.resolve(
    {
      source: filename,
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

async function resolveDependency(
  url: string,
  transformParam: PluginTransformHookParam,
  ctx: CompilationContext
) {
  if (!isAbsolute(url)) {
    const relPath = path.join(path.dirname(transformParam.resolvedPath), url);

    if (existsSync(relPath) && statSync(relPath).isFile()) {
      return relPath;
    }
  }

  const try_prefix_list = ['_'];
  let default_import_error;
  try {
    const result = await resolveDependencyWithPrefix(
      url,
      transformParam,
      '',
      ctx
    );
    if (result && (result.endsWith('.scss') || result.endsWith('.sass')))
      return result;
  } catch (error) {
    default_import_error = error;
  }

  for (const prefix of try_prefix_list) {
    try {
      const result = await resolveDependencyWithPrefix(
        url,
        transformParam,
        prefix,
        ctx
      );
      if (result) {
        return result;
      }
    } catch (_error) {
      /* do nothing */
    }
  }

  if (default_import_error) {
    throw default_import_error;
  }
}

const syntaxMap: Record<string, string> = {
  '.css': 'css',
  '.sass': 'indented'
};

function urlCanParse(file: string): boolean {
  try {
    return !!new URL(file);
  } catch (error) {
    return false;
  }
}

function normalizePath(file: string, root: string): string {
  if (urlCanParse(file)) {
    return normalizePath(fileURLToPath(new URL(file)), root);
  }

  if (path.isAbsolute(file)) {
    return file;
  }

  return path.relative(root, path.join(root, file));
}

async function compileScss(param: CompileCssParams) {
  const {
    transformParam,
    additionContext,
    sassImpl,
    sourceMapEnabled,
    options,
    ctx,
    root
  } = param;

  const { css, sourceMap, loadedUrls } = (await sassImpl.compileStringAsync(
    `${additionContext}\n${transformParam.content}`,
    {
      ...(options?.sassOptions ?? {}),
      sourceMap: options.sassOptions?.sourceMap ?? sourceMapEnabled,
      url: pathToFileURL(transformParam.resolvedPath),
      syntax: syntaxMap[path.extname(transformParam.moduleId)] ?? 'scss',
      importers: [
        {
          async canonicalize(url, _) {
            if (urlCanParse(url)) return new URL(url);
            // file:///xxxx
            // /xxx
            // ./xxx
            const normalizedPath = normalizePath(url, root);
            const normalizedUrl = path.relative(root, normalizedPath);
            const filePath = await resolveDependency(
              normalizedUrl.replaceAll('\\', '/'),
              transformParam,
              ctx
            );
            return pathToFileURL(filePath);
          },
          async load(canonicalUrl) {
            const filePath = fileURLToPath(canonicalUrl);
            const { contents } = await rebaseUrls(
              filePath,
              transformParam.resolvedPath,
              '$',
              (id, importer) => {
                return resolveDependency(
                  id,
                  {
                    ...transformParam,
                    moduleId: importer
                  },
                  ctx
                );
              }
            );
            return {
              contents: contents ?? (await readFile(filePath, 'utf-8')),
              syntax: syntaxMap[path.extname(filePath)] ?? 'scss',
              sourceMapUrl: canonicalUrl
            };
          }
        }
      ]
    } as StringOptions<'async'>
  )) as CompileResult;

  for (const fileUrl of loadedUrls) {
    const file = fileURLToPath(fileUrl);

    if (file === transformParam.resolvedPath) continue;

    ctx.addWatchFile(transformParam.resolvedPath, file);
  }

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
        includePaths: ['node_modules'],
        ...(options?.sassOptions ?? {}),
        data: `${additionContext}\n${transformParam.content}`,
        sourceMap: options.sassOptions?.sourceMap ?? sourceMapEnabled,
        outFile: transformParam.resolvedPath,
        sourceMapRoot: path.dirname(transformParam.resolvedPath),
        indentedSyntax: transformParam.moduleId.endsWith('.sass'),
        importer: [
          function (url, _, done) {
            resolveDependency(url, transformParam, ctx).then((resolvedPath) => {
              rebaseUrls(
                resolvedPath,
                transformParam.resolvedPath,
                '$',
                (id, importer) => {
                  return resolveDependency(
                    id,
                    {
                      ...transformParam,
                      moduleId: importer
                    },
                    ctx
                  );
                }
              ).then(({ contents }) => {
                done({ file: resolvedPath, contents });
              });
            });
          }
        ]
      },
      (err, result) => {
        if (err) {
          reject(err);
          return;
        }

        result.stats.includedFiles.forEach((file) => {
          if (file === transformParam.resolvedPath) return;
          ctx.addWatchFile(transformParam.resolvedPath, file);
        });

        resolve({
          css: result.css.toString(),
          sourceMap: result.map && result.map.toString()
        });
      }
    );
  });
}
