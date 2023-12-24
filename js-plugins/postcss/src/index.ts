import { JsPlugin, ResolvedUserConfig } from '@farmfe/core';
import postcssLoadConfig from 'postcss-load-config';
import { ProcessOptions, Processor } from 'postcss';
import path from 'path';
import glob from 'fast-glob';
import { getPostcssImplementation, pluginName } from './utils.js';

export type PostcssPluginOptions = {
  /**
   * @default undefined
   * postcss-load-config options. path default to farm.config.js root.
   */
  postcssLoadConfig?: {
    ctx?: postcssLoadConfig.ConfigContext;
    path?: string;
    options?: Parameters<typeof postcssLoadConfig>[2];
  };
  filters?: {
    resolvedPaths?: string[];
    moduleTypes?: string[];
  };
  implementation?: string;
};

export default function farmPostcssPlugin(
  options: PostcssPluginOptions = {}
): JsPlugin {
  let postcssProcessor: Processor;
  let postcssOptions: ProcessOptions;

  const implementation = getPostcssImplementation(options?.implementation);

  return {
    name: pluginName,
    // Execute last
    priority: 0,

    configResolved: async (config: ResolvedUserConfig) => {
      const { plugins, options: _options } = await postcssLoadConfig(
        options.postcssLoadConfig?.ctx,
        options.postcssLoadConfig?.path ?? config.root,
        options.postcssLoadConfig?.options
      );
      postcssOptions = _options;
      postcssProcessor = implementation(plugins);
    },

    transform: {
      filters: {
        resolvedPaths: options.filters?.resolvedPaths,
        moduleTypes: options.filters?.moduleTypes ?? ['css']
      },
      async executor(param, context) {
        try {
          const sourceMapEnabled = context.sourceMapEnabled(param.moduleId);

          const { css, map, messages } = await postcssProcessor.process(
            param.content,
            {
              ...postcssOptions,
              from: param.resolvedPath,
              map: sourceMapEnabled
            }
          );
          // record CSS dependencies from @imports
          if (process.env.NODE_ENV === 'development') {
            for (const message of messages) {
              if (message.type === 'dependency') {
                context.addWatchFile(
                  param.resolvedPath,
                  message.file as string
                );
              } else if (message.type === 'dir-dependency') {
                const { dir, glob: globPattern = '**' } = message;
                // https://github.com/postcss/postcss/blob/main/docs/guidelines/runner.md#3-dependencies
                const files = glob.sync(path.join(dir, globPattern), {
                  ignore: ['**/node_modules/**']
                });
                for (const file of files) {
                  context.addWatchFile(param.resolvedPath, file);
                }
              } else if (message.type === 'warning') {
                console.warn(`[${pluginName}] ${message.text}`);
              }
            }
          }
          return {
            content: css,
            moduleType: 'css',
            sourceMap: map && JSON.stringify(map.toJSON())
          };
        } catch (error) {
          context.error(`[${pluginName}] ${error}`);
        }

        return {
          content: '',
          moduleType: 'css'
        };
      }
    }
  };
}
