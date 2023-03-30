import fs from 'fs';
import { parse } from '@vue/compiler-sfc';
import { JsPlugin } from '@farmfe/core';
import { handleHmr } from './farm-vue-hmr.js';
import {
  StylesCodeCache,
  CacheDescriptor,
  LessStatic,
} from './farm-vue-types.js';
import { genMainCode } from './generatorCode.js';
import { callWithErrorHandle, error } from './utils.js';

//apply style langs
type ApplyStyleLangs = ['less'];

const stylesCodeCache: StylesCodeCache = {};
const applyStyleLangs = ['less'];
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
  try {
    switch (moduleType) {
      case 'less':
        let lessProcessor = (await import(moduleType)) || {};
        if (lessProcessor.default) {
          lessProcessor = lessProcessor.default;
        }
        return await transformLessToCss(styleCode, lessProcessor as LessStatic);
      default:
        return __default;
    }
  } catch (err) {
    error({ id: 'less', message: err });
  }
  return __default;
}

async function transformLessToCss(lessCode: string, lessProcessor: LessStatic) {
  return await lessProcessor.render(lessCode, {});
}
