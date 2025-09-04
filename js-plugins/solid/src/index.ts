import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { extname } from 'node:path';
import { transformSync } from '@babel/core';
import ts from '@babel/preset-typescript';
import { createFilter } from '@rollup/pluginutils';
import solid from 'babel-preset-solid';
import { mergeAndConcat } from 'merge-anything';
import solidRefresh from 'solid-refresh/babel';

import type { TransformOptions } from '@babel/core';
import { isObject, type JsPlugin } from '@farmfe/core';
import type { Options } from './types.js';

// TODO: HMR
const require = createRequire(import.meta.url);

const runtimePublicPath = '/@solid-refresh';
const runtimeFilePath = require.resolve('solid-refresh/dist/solid-refresh.mjs');
const runtimeCode = readFileSync(runtimeFilePath, 'utf-8');

function tryToReadFileSync(path: string) {
  try {
    return readFileSync(path, 'utf-8');
  } catch (error) {
    console.error(`[Farm Plugin Solid]: ${error.type}: ${error.message}`);
  }
}

export default function farmPluginSolid(
  options: Partial<Options> = {}
): JsPlugin {
  const filter = createFilter(options.include, options.exclude);

  let needHmr = false;
  let replaceDev = false;
  let projectRoot = process.cwd();

  const extensionsToWatch = [...(options.extensions ?? []), '.tsx', '.jsx'];
  const allExtensions = extensionsToWatch.map((extension) =>
    // An extension can be a string or a tuple [extension, options]
    typeof extension === 'string' ? extension : extension[0]
  );

  return {
    name: 'farm-plugin-solid',
    config(config) {
      return {
        compilation: {
          lazyCompilation:
            options.ssr === true ? false : config.compilation?.lazyCompilation
        },
        server: {
          hmr: options.ssr === true ? false : config.server?.hmr
        }
      };
    },
    configResolved(config) {
      const root = config.root ?? process.cwd();
      const mode = config.compilation?.mode;
      // We inject the dev mode only if the use˜r explicitly wants it or if we are in dev (serve) mode
      needHmr = mode !== 'production';
      replaceDev = options.dev === true || mode === 'development';
      projectRoot = root ?? process.cwd();

      if (!config.compilation.resolve) {
        config.compilation.resolve = {};
      }

      config.compilation.resolve.conditions = [
        ...(config.compilation.resolve.conditions ?? []),
        'solid',
        ...(replaceDev ? ['development'] : [])
      ];

      if (Array.isArray(config.compilation?.resolve?.alias)) {
        config.compilation.resolve.alias.push({ find: 'solid-refresh', replacement: runtimePublicPath })
      } else if (isObject(config.compilation?.resolve?.alias)) {
        config.compilation.resolve.alias['solid-refresh'] = runtimePublicPath;
      } else {
        config.compilation.resolve.alias = [
          { find: 'solid-refresh', replacement: runtimePublicPath }
        ]
      }
    },
    load: {
      filters: {
        resolvedPaths: [...allExtensions, runtimePublicPath]
      },
      async executor(param) {
        if (param.resolvedPath === runtimePublicPath) {
          return {
            content: runtimeCode,
            moduleType: 'solid-refresh'
          };
        }

        const source = tryToReadFileSync(param.resolvedPath);

        return {
          content: source,
          moduleType: 'solid'
        };
      }
    },
    transform: {
      filters: {
        moduleTypes: ['solid', 'solid-refresh']
      },
      async executor(param) {
        const isSsr = options.ssr;
        const currentFileExtension = extname(param.resolvedPath);

        if (
          !filter(param.resolvedPath) ||
          !allExtensions.includes(currentFileExtension)
        ) {
          return;
        }

        const inNodeModules = /node_modules/.test(param.resolvedPath);

        let solidOptions: { generate: 'ssr' | 'dom'; hydratable: boolean };

        if (options.ssr) {
          if (isSsr) {
            solidOptions = { generate: 'ssr', hydratable: true };
          } else {
            solidOptions = { generate: 'dom', hydratable: true };
          }
        } else {
          solidOptions = { generate: 'dom', hydratable: false };
        }

        param.resolvedPath = param.resolvedPath.replace(/\?.+$/, '');

        const opts: TransformOptions = {
          babelrc: false,
          configFile: false,
          root: projectRoot,
          filename: param.resolvedPath,
          sourceFileName: param.resolvedPath,
          presets: [[solid, { ...solidOptions, ...(options.solid ?? {}) }]],
          plugins:
            needHmr && !isSsr && !inNodeModules
              ? [[solidRefresh, { bundler: 'standard' }]]
              : [],
          sourceMaps: true,
          // Vite handles sourcemap flattening
          inputSourceMap: false as any
        };

        // We need to know if the current file extension has a typescript options tied to it
        const shouldBeProcessedWithTypescript = extensionsToWatch.some(
          (extension) => {
            if (typeof extension === 'string') {
              return extension.includes('tsx');
            }

            const [extensionName, extensionOptions] = extension;
            if (extensionName !== currentFileExtension) return false;

            return extensionOptions.typescript;
          }
        );

        if (shouldBeProcessedWithTypescript) {
          opts.presets.push([ts, options.typescript ?? {}]);
        }

        // Default value for babel user options
        let babelUserOptions: TransformOptions = {};

        if (options.babel) {
          if (typeof options.babel === 'function') {
            const babelOptions = options.babel(
              param.content,
              param.resolvedPath,
              isSsr
            );
            babelUserOptions =
              babelOptions instanceof Promise
                ? await babelOptions
                : babelOptions;
          } else {
            babelUserOptions = options.babel;
          }
        }

        const babelOptions = mergeAndConcat(
          babelUserOptions,
          opts
        ) as TransformOptions;

        const { code = '', map = {} } = transformSync(
          param.content,
          babelOptions
        );

        return {
          content: code,
          sourceMap: JSON.stringify(map),
          moduleType: 'js'
        };
      }
    }
  };
}
