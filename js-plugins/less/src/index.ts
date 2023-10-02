import { JsPlugin, UserConfig, DevServer } from '@farmfe/core';
import {
  getLessImplementation,
  tryRead,
  pluginName,
  throwError
} from './utils.js';
import path from 'path';

export type LessPluginOptions = Less.Options & {
  implementation?: string;
  sourceMap?: boolean;
  additionalData?:
    | string
    | ((context?: string, resolvePath?: string) => string | Promise<string>);
};
export default function farmLessPlugin(
  options: LessPluginOptions = {}
): JsPlugin {
  let farmConfig: UserConfig['compilation'];
  let devServer: DevServer;
  const implementation = getLessImplementation(options?.implementation);
  return {
    name: pluginName,
    config: (param) => (farmConfig = param),
    configDevServer(server) {
      devServer = server;
    },
    load: {
      filters: { resolvedPaths: ['\\.less$'] },
      async executor(param) {
        const data = await tryRead(param.resolvedPath);
        return {
          content: data,
          moduleType: 'less'
        };
      }
    },
    transform: {
      filters: { resolvedPaths: ['\\.less$'] },
      async executor(param) {
        try {
          const isProd = farmConfig.mode === 'production';
          let relData = '';
          const fileRoot = path.dirname(param.resolvedPath);
          const configPaths = options.paths;
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

          const { css, sourceMap, imports } = await implementation.render(
            relData,
            {
              ...options,
              sourceMap: {
                outputSourceFiles: Boolean(
                  options.sourceMap ?? farmConfig?.sourcemap
                )
              },
              paths: configPaths ? [fileRoot, ...configPaths] : [fileRoot]
            }
          );
          if (imports && !isProd) {
            for (const dep of imports) {
              devServer.addWatchFile(param.resolvedPath, [
                path.resolve(fileRoot, dep)
              ]);
            }
          }
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
