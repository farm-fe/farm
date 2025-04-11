import fs from 'fs';
import path from 'path';
import type { JsPlugin, UserConfig } from '@farmfe/core';
import { parse } from '@vue/compiler-sfc';
import { compileStyle } from '@vue/compiler-sfc';
import { handleHmr } from './farm-vue-hmr.js';
import {
  CacheDescriptor,
  FarmVuePluginOptions,
  PreProcessors,
  PreProcessorsOptions,
  PreProcessorsType,
  StylesCodeCache,
  ValueOf
} from './farm-vue-types.js';
import { genMainCode } from './generatorCode.js';
import {
  callWithErrorHandle,
  error,
  getHash,
  getResolvedOptions,
  handleExclude,
  handleInclude,
  isLess,
  isSass,
  isStyl,
  loadPreProcessor
} from './utils.js';

const stylesCodeCache: StylesCodeCache = {};
const applyStyleLangs = ['less', 'sass', 'scss', 'stylus'];
const cacheDescriptor: CacheDescriptor = {};

const parseQuery = (query: [string, string][]) =>
  query.reduce(
    (pre, [key, value]) => {
      pre[key] = value;

      return pre;
    },
    {} as Record<string, string>
  );

export default function farmVuePlugin(
  farmVuePluginOptions: FarmVuePluginOptions = {}
): JsPlugin {
  // options hooks to get farmConfig
  let farmConfig: UserConfig['compilation'];
  const resolvedOptions = getResolvedOptions(farmVuePluginOptions);
  const exclude = handleExclude(resolvedOptions);
  const include = handleInclude(resolvedOptions);

  return {
    name: 'farm-vue-plugin',
    config(config) {
      return {
        compilation: {
          lazyCompilation:
            resolvedOptions.ssr === true
              ? false
              : config.compilation?.lazyCompilation
        },
        server: {
          hmr: resolvedOptions.hmr ?? config.server?.hmr
        }
      };
    },
    configResolved(config) {
      farmConfig = config.compilation || {};
    },
    load: {
      filters: {
        resolvedPaths: ['\\.vue($|\\?)', ...include]
      },
      async executor(params, ctx, hookContext) {
        const { resolvedPath } = params;
        let source = '';

        const query = parseQuery(params.query);
        const { vue, lang, hash, scoped, index } = query;

        // handle .vue file
        if (vue === 'true' && hash) {
          let cssCode = stylesCodeCache[hash];
          // if lang is not "css", use preProcessor to handle
          if (applyStyleLangs.includes(lang)) {
            const { css } = await preProcession(cssCode, lang, {
              paths: [path.dirname(resolvedPath)]
            });
            cssCode = css;
          }
          const descriptor = cacheDescriptor[resolvedPath];
          const block = descriptor.styles[Number(index)];

          const { code: styleCode, errors } = compileStyle({
            source: cssCode,
            id: `data-v-${scoped ?? getHash(resolvedPath)}`,
            scoped: block.scoped,
            filename: descriptor.filename,
            isProd: farmConfig.mode === 'production',
            // preprocessLang: lang !== 'css' ? lang as 'less' | 'sass' | 'scss' | 'stylus' : undefined,
            // preprocessCustomRequire: loadPreProcessor,
            ...resolvedOptions.style
          });

          if (errors.length) {
            errors.forEach((err) => {
              error({ id: err.name, message: err.message });
            });
            return;
          }
          return {
            content: styleCode,
            moduleType: 'css'
          };
        }

        try {
          source = await fs.promises.readFile(resolvedPath, 'utf-8');
        } catch (err) {
          error({
            id: resolvedPath,
            message:
              "path is not right, can't readFile" +
              JSON.stringify(ctx) +
              JSON.stringify(hookContext)
          });
        }
        return {
          content: source,
          moduleType: 'vue'
        };
      }
    },
    // add hmr code in root file
    transform: {
      filters: {
        moduleTypes: ['vue']
      },
      async executor(params, ctx) {
        try {
          // If path in exclude,skip transform.
          for (const reg of exclude) {
            if (reg.test(params.resolvedPath)) {
              return { content: params.content, moduleType: params.moduleType };
            }
          }

          const query = parseQuery(params.query);
          const { resolvedPath, content: source } = params;

          // transform vue
          const result = callWithErrorHandle<null, typeof parse, [string]>(
            this,
            parse,
            [source]
          );

          if (result) {
            const { descriptor } = result;

            const enableHMR =
              resolvedOptions.hmr && farmConfig.mode !== 'production';

            const beforeDescriptor = cacheDescriptor[resolvedPath];
            // set descriptors cache to hmr
            if (!beforeDescriptor) {
              if (Object.keys(query).length === 0)
                cacheDescriptor[resolvedPath] = descriptor;
            } else if (enableHMR) {
              const isHmr = handleHmr(
                resolvedOptions,
                beforeDescriptor,
                descriptor,
                stylesCodeCache,
                resolvedPath
              );
              if (isHmr) {
                return {
                  content: isHmr.source,
                  moduleType: isHmr.moduleType,
                  sourceMap: isHmr.map
                };
              }
            }

            const {
              source: mainCode,
              moduleType,
              map
            } = genMainCode(
              resolvedOptions,
              descriptor,
              stylesCodeCache,
              resolvedPath
            );
            return {
              content: mainCode,
              moduleType,
              sourceMap: map
            };
          } // default
          else {
            console.error(
              `[farm-vue-plugin]:there is no path can be match,please check!`
            );
            return {
              content:
                'console.log(`[farm-vue-plugin]:error:there is no path can be match,please check!`)',
              moduleType: 'js'
            };
          }
        } catch (err) {
          console.error(err);
          throw err;
        }
      }
    }
  };
}

async function preProcession(
  styleCode: string,
  moduleType: string,
  options?: { paths: string[] }
) {
  const __default = { css: styleCode, map: '' };
  let processor: ValueOf<PreProcessors>;
  try {
    // load less/sass/stylus preprocessor
    // compile style code to css
    switch (moduleType) {
      case 'less':
        processor = await loadPreProcessor(PreProcessorsType.less);
        return await compilePreProcessorCodeToCss(styleCode, processor, {
          paths: options.paths ?? []
        });
      case 'sass':
      case 'scss':
        processor = await loadPreProcessor(PreProcessorsType.sass);
        return await compilePreProcessorCodeToCss(styleCode, processor, {
          // @ts-ignore
          indentedSyntax: moduleType === 'sass',
          includePaths: options.paths ?? []
        });
      case 'stylus':
        processor = await loadPreProcessor(PreProcessorsType.stylus);
        return await compilePreProcessorCodeToCss(styleCode, processor, {
          paths: options.paths ?? []
        });
      default:
        return __default;
    }
  } catch (err) {
    error({ id: moduleType, message: err });
  }
  return __default;
}

export async function compilePreProcessorCodeToCss<
  T extends ValueOf<PreProcessors>
>(
  styleCode: string,
  preProcessor: T,
  options?: PreProcessorsOptions<T>
): Promise<{ css: string }> {
  if (isLess(preProcessor)) {
    return await new Promise((resolve, reject) => {
      preProcessor.render(
        styleCode,
        options as Less.Options,
        (error, { css }) => {
          if (error) {
            reject(error);
          }

          resolve({ css });
        }
      );
    });
  }

  if (isSass(preProcessor)) {
    return await new Promise((resolve, reject) => {
      preProcessor.render(
        {
          data: styleCode,
          ...((options as PreProcessorsOptions<
            PreProcessors[PreProcessorsType.sass]
          >) ?? {})
        },
        (exception, { css }) => {
          if (exception) {
            reject(exception);
          }

          resolve({ css: css.toString() });
        }
      );
    });
  }

  if (isStyl(preProcessor)) {
    return await new Promise((resolve, reject) => {
      preProcessor.render(styleCode, options, (err, css) => {
        if (err) {
          reject(err);
        }

        resolve({ css });
      });
    });
  }
}
