import type { JsPlugin } from '@farmfe/core';

import Context from './context.js';
import { tryToReadFileSync } from './utils.js';
import { pluginName } from './options.js';

import type { DtsPluginOptions } from './types.js';

export default function farmDtsPlugin(options: DtsPluginOptions): JsPlugin {
  const ctx = new Context();
  // TODO support vue other framework file type
  return {
    name: pluginName,
    priority: 1000,
    configResolved(config) {
      ctx.handleResolveOptions(options, config.compilation);
    },
    load: {
      filters: {
        resolvedPaths: ['.ts$']
      },
      async executor(params) {
        const { resolvedPath } = params;
        const content = await tryToReadFileSync(resolvedPath);
        return {
          content,
          moduleType: 'dts'
        };
      }
    },
    transform: {
      filters: {
        moduleTypes: ['dts']
      },
      async executor(params) {
        const { resolvedPath, content } = params;
        ctx.handleTransform(resolvedPath);
        return {
          content,
          moduleType: 'ts'
        };
      }
    },
    finish: {
      executor() {
        ctx.handleCloseBundle();
      }
    }
  };
}
