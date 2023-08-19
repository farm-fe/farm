import type { JsPlugin } from '@farmfe/core';

import Context from './context.js';
import { tryToReadFileSync } from './utils.js';
export default function farmDtsPlugin(options: any = {}): JsPlugin {
  const ctx = new Context();
  // TODO support vue other file type
  return {
    name: 'farm-dts-plugin',
    priority: 1000,
    config(config: any) {
      ctx.handleResolveOptions(options, config);
      return config;
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
