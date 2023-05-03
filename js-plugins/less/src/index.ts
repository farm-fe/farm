import { JsPlugin, UserConfig } from '@farmfe/core';
import { pluginName } from './options.js';
import { getLessImplementation, tryRead } from './utils.js';

export type LessPluginOptions = Less.Options & {
  implementation?: string;
  sourceMap?: boolean;
  additionalData?: string | Function;
};
export default function farmLessPlugin(options?: LessPluginOptions): JsPlugin {
  let farmConfig: UserConfig;
  const implementation = getLessImplementation(options?.implementation);
  return {
    name: pluginName,
    config: (param) => (farmConfig = param),
    load: {
      filters: { resolvedPaths: ['\\.less$'] },
      async executor(param) {
        const data = await tryRead(param.resolvedPath);
        return {
          content: data,
          moduleType: 'sass',
        };
      },
    },
    transform: {
      filters: { resolvedPaths: ['\\.less$'] },
      async executor(param) {
        try {
          let relData;
          if (
            typeof options.additionalData !== 'undefined' &&
            options.additionalData
          ) {
            relData =
              typeof options.additionalData === 'function'
                ? `${await options.additionalData(param.content, this)}`
                : `${options.additionalData}\n${param.content}`;
          }
          const { css, sourceMap } = await implementation.render(relData, {
            sourceMap: {
              outputSourceFiles: Boolean(
                options.sourceMap ?? farmConfig?.compilation?.sourcemap
              ),
            },
            ...options,
          });
          return {
            content: css,
            moduleType: 'css',
            sourceMap: sourceMap && JSON.stringify(sourceMap),
          };
        } catch (e) {
          console.error(e);
        }
        return {
          content: '',
          moduleType: 'css',
        };
      },
    },
  };
}
