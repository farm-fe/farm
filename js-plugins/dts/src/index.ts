import { JsPlugin, UserConfig } from '@farmfe/core';
import { getResolvedOptions, handleExclude, handleInclude } from './utils';
export default function farmDtsPlugin(
  farmDtsPluginOptions: any = {}
): JsPlugin {
  // options hooks to get farmConfig
  let farmConfig: UserConfig['compilation'];
  const resolvedOptions = getResolvedOptions(farmDtsPluginOptions);

  const exclude = handleExclude(resolvedOptions);
  const include = handleInclude(resolvedOptions);
  return {
    name: 'farm-dts-plugin',
    config(config: any) {
      farmConfig = config || {};
      return config;
    },
    load: {
      filters: {
        resolvedPaths: ['.ts$']
      },
      async executor(params: any, ctx: any) {
        const { resolvedPath } = params;
        console.log(resolvedPath);

        console.log(ctx);

        let source = '';
        return {
          content: source,
          moduleType: 'ts'
        };
      }
    },
    // add hmr code In root file
    transform: {
      filters: {
        // resolvedPaths: ['.ts$', ...include]
        resolvedPaths: ['.ts$']
      },
      async executor(params: any, ctx: any) {
        const { resolvedPath } = params;
        console.log(resolvedPath);

        console.log(ctx);

        let source = '';
        return {
          content: source,
          moduleType: 'ts'
        };
      }
    }
  };
}
