import { JsPlugin, UserConfig } from '@farmfe/core';
import postcssLoadConfig, { ProcessOptionsPreload } from 'postcss-load-config';
import { ProcessOptions, Processor } from 'postcss';
import path from 'path';
import glob from 'fast-glob';
import { getPostcssImplementation, pluginName } from './utils.js';

export type PostcssPluginOptions = ProcessOptionsPreload & {
  sourceMap?: boolean;
  implementation?: string;
};

export default function farmPostcssPlugin(
  options: PostcssPluginOptions = {}
): JsPlugin {
  let farmConfig: UserConfig['compilation'];
  let postcssProcessor: Processor;
  let postcssOptions: ProcessOptions;

  const implementation = getPostcssImplementation(options?.implementation);

  return {
    name: pluginName,
    // Execute last
    priority: 0,

    config: async (config: UserConfig) => {
      const { plugins, options: _options } = await postcssLoadConfig(
        options,
        config.root
      );
      postcssOptions = _options;
      postcssProcessor = implementation(plugins);
      farmConfig = config;
      return config;
    },

    transform: {
      filters: { moduleTypes: ['css'] },
      async executor(param, context) {
        try {
          const { css, map, messages } = await postcssProcessor.process(
            param.content,
            {
              ...postcssOptions,
              from: param.resolvedPath,
              map: Boolean(options.sourceMap ?? farmConfig?.sourcemap)
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
            sourceMap: options.sourceMap && JSON.stringify(map.toJSON())
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
