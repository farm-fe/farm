import { DevServer, JsPlugin, UserConfig } from '@farmfe/core';
import {
  getLessImplementation,
  pluginName,
  throwError,
  tryRead
} from './utils.js';
import path from 'path';
import { existsSync } from 'fs';

export type LessPluginOptions = {
  lessOptions?: Less.Options;
  implementation?: string;
  filters?: {
    resolvedPaths?: string[];
    moduleTypes?: string[];
  };
  additionalData?:
    | string
    | ((content?: string, resolvePath?: string) => string | Promise<string>);
};

export default function farmLessPlugin(
  options: LessPluginOptions = {}
): JsPlugin {
  let farmConfig: UserConfig['compilation'];
  let devServer: DevServer;
  const implementation: LessStatic = getLessImplementation(
    options?.implementation
  );

  return {
    name: pluginName,
    config: (config) => {
      farmConfig = config;
      return config;
    },
    configDevServer(server) {
      devServer = server;
    },
    load: {
      filters: {
        resolvedPaths: options.filters?.resolvedPaths ?? ['\\.less$']
      },
      async executor(param) {
        if (param.query.length === 0 && existsSync(param.resolvedPath)) {
          const data = await tryRead(param.resolvedPath);

          return {
            content: data,
            moduleType: 'less'
          };
        }

        return null;
      }
    },
    transform: {
      filters: {
        resolvedPaths: options.filters?.resolvedPaths,
        moduleTypes: options.filters?.moduleTypes ?? ['less']
      },
      async executor(param, ctx) {
        try {
          const isProd = farmConfig.mode === 'production';
          let relData = '';
          const fileRoot = path.dirname(param.resolvedPath);
          const configPaths = options.lessOptions?.paths;
          if (
            typeof options.additionalData !== 'undefined' &&
            options.additionalData
          ) {
            relData =
              typeof options.additionalData === 'function'
                ? `${await options.additionalData(
                    param.content,
                    param.resolvedPath
                  )}`
                : `${options.additionalData}\n${param.content}`;
            //  If the additionalData is a function, it might be return null or undefined, so we need to check it
            if (typeof relData !== 'string') {
              relData = param.content;
            }
          } else {
            relData = param.content;
          }

          const sourceMapEnabled = ctx.sourceMapEnabled(param.moduleId);

          const { css, map, imports } = await implementation.render(relData, {
            ...(options?.lessOptions ?? {}),
            filename: param.resolvedPath,
            sourceMap:
              (options.lessOptions?.sourceMap ?? sourceMapEnabled) && {},
            paths: configPaths ? [fileRoot, ...configPaths] : [fileRoot]
          } as Less.Options);

          if (devServer && imports && !isProd) {
            for (const dep of imports) {
              // TODO add a compilerCreated hook to farmfe/core and get the compiler instead of using devServer
              devServer.addWatchFile(param.resolvedPath, [
                path.resolve(fileRoot, dep)
              ]);
            }
          }
          return {
            content: css,
            moduleType: 'css',
            sourceMap: map
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
