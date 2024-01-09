/**
 * Lazy compilation middleware. Using the same logic as HMR middleware, but
 */

import { relative } from 'node:path';
import { Context, Middleware } from 'koa';

import { DevServer } from '../index.js';
import { bold, cyan, green } from '../../index.js';

import type { Resource } from '@farmfe/runtime/src/resource-loader.js';
import { existsSync } from 'node:fs';

export function lazyCompilation(devSeverContext: DevServer): Middleware {
  const compiler = devSeverContext.getCompiler();

  if (!compiler.config.config?.lazyCompilation) {
    return;
  }

  return async (ctx: Context, next: () => Promise<any>) => {
    if (ctx.path === '/__lazy_compile') {
      const paths = (ctx.query.paths as string).split(',');
      const pathsStr = paths
        .map((p) => {
          if (p.startsWith('/') && !existsSync(p)) {
            return p;
          }
          const resolvedPath = compiler.transformModulePath(
            compiler.config.config.root,
            p
          );
          return relative(compiler.config.config.root, resolvedPath);
        })
        .join(', ');
      devSeverContext.logger.info(`Lazy compiling ${bold(cyan(pathsStr))}`);
      const start = Date.now();
      const result = await compiler.update(paths);
      devSeverContext.logger.info(
        `${bold(green(`✓`))} Lazy compilation done in ${bold(
          green(`${Date.now() - start}ms`)
        )}.`
      );

      devSeverContext.hmrEngine.callUpdates(result);

      if (result) {
        let dynamicResourcesMap: Record<string, Resource[]> = null;

        if (result.dynamicResourcesMap) {
          for (const [key, value] of Object.entries(
            result.dynamicResourcesMap
          )) {
            if (!dynamicResourcesMap) {
              dynamicResourcesMap = {} as Record<string, Resource[]>;
            }

            dynamicResourcesMap[key] = value.map((r) => ({
              path: r[0],
              type: r[1] as 'script' | 'link'
            }));
          }
        }

        const code = `export default {
          immutableModules: ${JSON.stringify(result.immutableModules.trim())},
          mutableModules: ${JSON.stringify(result.mutableModules.trim())},
          dynamicResourcesMap: ${JSON.stringify(dynamicResourcesMap)}
        }`;
        ctx.type = 'application/javascript';
        ctx.body = code;
      } else {
        throw new Error(`Lazy compilation result not found for paths ${paths}`);
      }
    } else {
      await next();
    }
  };
}
