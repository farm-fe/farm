import path from 'node:path';
import type { JsPlugin } from 'farm';

import Context from './context.js';
import { pluginName } from './options.js';
import { tryToReadFileSync } from './utils.js';

import type { DtsPluginOptions } from './types.js';

const extension = ['.ts', '.tsx'].map((ext) => `${ext}$`);

export default function farmDtsPlugin(options?: DtsPluginOptions): JsPlugin {
  const ctx = new Context();

  return {
    name: pluginName,
    priority: 1000,
    configResolved(config) {
      ctx.handleResolveOptions(options, config.compilation);
    },
    load: {
      filters: {
        resolvedPaths: [
          ...(Array.isArray(options?.resolvedPaths)
            ? options.resolvedPaths
            : extension)
        ]
      },
      async executor(params) {
        const { resolvedPath } = params;
        const loadFileExtName = path.extname(resolvedPath);
        const isTypescriptFile = extension.some((ext) =>
          new RegExp(ext).test(loadFileExtName)
        );

        if (!isTypescriptFile) {
          return null;
        }
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
        const [url] = resolvedPath.split('?');
        ctx.handleTransform(resolvedPath, content);

        const ext = path.extname(url).slice(1);

        return {
          content,
          moduleType: ext || 'ts'
        };
      }
    },
    finish: {
      async executor() {
        ctx.handleCloseBundle();
      }
    },
    updateFinished: {
      async executor() {
        ctx.handleCloseBundle();
      }
    }
  };
}
