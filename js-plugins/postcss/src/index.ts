import path from 'path';
import { JsPlugin, ResolvedUserConfig, checkPublicFile } from '@farmfe/core';
import glob from 'fast-glob';
import { ProcessOptions, Processor } from 'postcss';
import postcssLoadConfig from 'postcss-load-config';
import { getPostcssImplementation, pluginName, tryRead } from './utils.js';
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
  internalPlugins?: {
    /**
     * @default false
     * @description please see https://www.npmjs.com/package/postcss-import
     */
    postcssImport?: boolean;
  };
};

export default function farmPostcssPlugin(
  options: PostcssPluginOptions = {}
): JsPlugin {
  let postcssProcessor: Processor;
  let postcssOptions: ProcessOptions;
  let postcssPlugins: postcssLoadConfig.ResultPlugin[] = [];
  let userConfig: ResolvedUserConfig;

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
      postcssPlugins = plugins;
      userConfig = config;
    },

    transform: {
      filters: {
        resolvedPaths: options.filters?.resolvedPaths,
        moduleTypes: options.filters?.moduleTypes ?? ['css']
      },
      async executor(param, context) {
        try {
          const sourceMapEnabled = context.sourceMapEnabled(param.moduleId);
          const enablePostcssImport =
            options.internalPlugins?.postcssImport ?? false;

          if (enablePostcssImport) {
            const atImport = getPostcssImplementation('postcss-import');
            const postcssUrl = getPostcssImplementation('postcss-url');

            postcssPlugins.unshift(
              atImport({
                resolve: async (id: string, basedir: string) => {
                  const publicFile = await checkPublicFile(id, userConfig);
                  if (publicFile) {
                    return publicFile;
                  }

                  try {
                    const resolvedInfo = await context.resolve(
                      {
                        kind: 'import',
                        importer: path.resolve(basedir, '*'),
                        source: id
                      },
                      {
                        meta: param.meta,
                        caller: pluginName
                      }
                    );
                    if (resolvedInfo.resolvedPath) {
                      return path.resolve(resolvedInfo.resolvedPath);
                    }
                  } catch {
                    // context.resolve will throw an error if it doesn't resolve, so it needs to be wrapped in a try block here.
                  }
                  if (!path.isAbsolute(id)) {
                    console.error(
                      `Unable to resolve \`@import "${id}"\` from ${basedir}`
                    );
                  }

                  return id;
                },
                load: async (id: string) => {
                  // After postcss-import inline process, the `url()` relative paths in the css file need to be recomputed relative to the entry CSS file.
                  const content = await tryRead(id);
                  const implementation = getPostcssImplementation();
                  const urlRebasePostcssProcessor: Processor = implementation([
                    postcssUrl({
                      url: 'rebase'
                    })
                  ]);
                  const { css } = await urlRebasePostcssProcessor.process(
                    content,
                    {
                      from: id,
                      to: param.resolvedPath
                    }
                  );
                  return css;
                }
              })
            );
          }
          postcssProcessor = implementation(postcssPlugins);

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
                context.addWatchFile(param.moduleId, message.file as string);
              } else if (message.type === 'dir-dependency') {
                const { dir, glob: globPattern = '**' } = message;
                // https://github.com/postcss/postcss/blob/main/docs/guidelines/runner.md#3-dependencies
                const files = glob.sync(globPattern, {
                  ignore: ['**/node_modules/**'],
                  cwd: dir
                });
                for (const file of files) {
                  context.addWatchFile(
                    param.moduleId,
                    path.isAbsolute(file) ? file : path.join(dir, file)
                  );
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
