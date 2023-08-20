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
  let farmConfig: UserConfig;
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
          let relData;
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
          } else {
            relData = param.content;
          }

          const { css, sourceMap, imports } = await implementation.render(
            relData,
            {
              ...options,
              sourceMap: {
                outputSourceFiles: Boolean(
                  options.sourceMap ?? farmConfig?.compilation?.sourcemap
                )
              },
              paths: configPaths ? [fileRoot, ...configPaths] : [fileRoot]
            }
          );
          if (imports) {
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
