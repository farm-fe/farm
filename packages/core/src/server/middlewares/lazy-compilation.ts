/**
 * Lazy compilation middleware. Using the same logic as HMR middleware
 */

import { relative } from 'node:path';
import { Context, Middleware, Next } from 'koa';

import { Server } from '../index.js';
import { bold, clearScreen, cyan, green } from '../../index.js';

import type { Resource } from '@farmfe/runtime/src/resource-loader.js';
import { existsSync } from 'node:fs';
import { logError } from '../error.js';

export function lazyCompilation(devSeverContext: Server): Middleware {
  const compiler = devSeverContext.getCompiler();

  if (!compiler.config.config?.lazyCompilation) {
    return;
  }

  return async (ctx: Context, next: Next) => {
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
      clearScreen();
      devSeverContext.logger.info(`Lazy compiling ${bold(cyan(pathsStr))}`);
      const start = Date.now();
      // sync update when node is true
      let result;
      try {
        result = await compiler.update(paths, Boolean(ctx.query.node));
      } catch (e) {
        logError(e);
      }

      if (!result) {
        return;
      }

      if (ctx.query.node) {
        compiler.writeResourcesToDisk();
      }

      devSeverContext.logger.info(
        `${bold(green(`✓`))} Lazy compilation done(${bold(
          cyan(pathsStr)
        )}) in ${bold(green(`${Date.now() - start}ms`))}.`
      );

      devSeverContext.hmrEngine?.callUpdates?.(result);

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

        const code = !ctx.query.node
          ? `export default {
          immutableModules: ${JSON.stringify(result.immutableModules.trim())},
          mutableModules: ${JSON.stringify(result.mutableModules.trim())},
          dynamicResourcesMap: ${JSON.stringify(dynamicResourcesMap)}
        }`
          : `{
          "immutableModules": ${JSON.stringify(result.immutableModules.trim())},
          "mutableModules": ${JSON.stringify(result.mutableModules.trim())},
          "dynamicResourcesMap": ${JSON.stringify(dynamicResourcesMap)}
        }`;
        ctx.type = !ctx.query.node
          ? 'application/javascript'
          : 'application/json';
        ctx.body = code;
      } else {
        throw new Error(`Lazy compilation result not found for paths ${paths}`);
      }
    } else {
      await next();
    }
  };
}
