import fs from 'fs';
import { parse } from '@vue/compiler-sfc';
import { JsPlugin } from '@farmfe/core';
import { handleHmr } from './farm-vue-hmr.js';
import {
  StylesCodeCache,
  CacheDescriptor,
  PreProcessors,
  PreProcessorsType,
  PreProcessorsOptions,
  ValueOf,
} from './farm-vue-types.js';
import { genMainCode } from './generatorCode.js';
import {
  callWithErrorHandle,
  error,
  isLess,
  isSass,
  isStyl,
  loadPreProcessor,
} from './utils.js';

//apply style langs
type ApplyStyleLangs = ['less', 'sass', 'scss', 'stylus'];

const stylesCodeCache: StylesCodeCache = {};
const applyStyleLangs = ['less', 'sass', 'scss', 'stylus'];
const cacheDescriptor: CacheDescriptor = {};

export default function farmVuePlugin(options: object = {}): JsPlugin {
  //options hooks to get farmConfig
  let farmConfig = null;
  return {
    name: 'farm-vue-plugin',
    load: {
      filters: {
        resolvedPaths: ['.vue$'],
      },
      async executor(params, ctx) {
        const { resolvedPath } = params;
        let source = '';
        try {
          source = await fs.promises.readFile(resolvedPath, 'utf-8');
        } catch (err) {
          error({
            id: resolvedPath,
            message: "path is not right,can't readFile",
          });
        }
        return {
          content: source,
          moduleType: 'ts',
        };
      },
    },
    // add hmr code In root file
    transform: {
      filters: {
        resolvedPaths: ['.vue$'],
      },
      async executor(params, ctx) {
        const query: Record<string, string> = {};
        params.query.forEach(([key, value]) => {
          query[key] = value;
        });
        const { vue, lang, hash } = query;
        const { resolvedPath, content: source } = params;
        //handle .vue file
        if (vue === 'true' && hash) {
          let styleCode = stylesCodeCache[hash];
          //if lang is not "css",use preProcessor to handle
          if (applyStyleLangs.includes(lang)) {
            const { css } = await preProcession(styleCode, lang);
            styleCode = css;
          }
          return {
            content: typeof styleCode === 'string' ? styleCode : '',
            moduleType: 'css',
          };
        }

        //transform vue
        const result = callWithErrorHandle<null, typeof parse, [string]>(
          this,
          parse,
          [source]
        );
        if (result) {
          const { descriptor } = result;
          const isHmr = handleHmr(
            cacheDescriptor,
            descriptor,
            stylesCodeCache,
            query,
            resolvedPath
          );
          if (isHmr)
            return {
              content: isHmr.source,
              moduleType: isHmr.moduleType,
              sourceMap: isHmr.map,
            };

          const {
            source: mainCode,
            moduleType,
            map,
          } = genMainCode(descriptor, stylesCodeCache, resolvedPath);
          return {
            content: mainCode,
            moduleType,
            sourceMap: map,
          };
        }

        //default
        else {
          console.error(
            `[farm-vue-plugin]:there is no path can be match,please check!`
          );
          return {
            content:
              'console.log(`[farm-vue-plugin]:error:there is no path can be match,please check!`)',
            moduleType: 'js',
          };
        }
        return {
          content: params.content,
          moduleType: params.moduleType,
        };
      },
    },
  };
}

async function preProcession(styleCode: string, moduleType: string) {
  const __default = { css: styleCode, map: '' };
  let processor: ValueOf<PreProcessors>;
  try {
    switch (moduleType) {
      case 'less':
        processor = await loadPreProcessor(PreProcessorsType.less);
        return await compilePreProcessorCodeToCss(styleCode, processor);
      case 'sass':
        processor = await loadPreProcessor(PreProcessorsType.sass);
        return await compilePreProcessorCodeToCss(styleCode, processor, {
          indentedSyntax: true,
        });
      case 'scss':
        processor = await loadPreProcessor(PreProcessorsType.sass);
        return await compilePreProcessorCodeToCss(styleCode, processor);
      case 'stylus':
        processor = await loadPreProcessor(PreProcessorsType.stylus);
        return await compilePreProcessorCodeToCss(styleCode, processor);
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
      preProcessor.render(styleCode, {}, (error, { css }) => {
        if (error) {
          reject(error);
        }

        resolve({ css });
      });
    });
  }

  if (isSass(preProcessor)) {
    return await new Promise((resolve, reject) => {
      preProcessor.render(
        {
          data: styleCode,
          ...((options as PreProcessorsOptions<
            PreProcessors[PreProcessorsType.sass]
          >) ?? {}),
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
      preProcessor.render(styleCode, {}, (err, css) => {
        if (err) {
          reject(err);
        }

        resolve({ css });
      });
    });
  }
}
