/**
 * Lazy compilation middleware. Using the same logic as HMR middleware
 */

import { relative } from 'node:path';
import { Context, Middleware, Next } from 'koa';

import {
  VIRTUAL_FARM_DYNAMIC_IMPORT_SUFFIX,
  bold,
  checkClearScreen,
  cyan,
  green
} from '../../index.js';
import { Server } from '../index.js';

import { existsSync } from 'node:fs';
import type { Resource } from '@farmfe/runtime/src/resource-loader.js';
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
          if (
            p.startsWith('/') &&
            !p.endsWith(VIRTUAL_FARM_DYNAMIC_IMPORT_SUFFIX) &&
            !existsSync(p)
          ) {
            return p;
          }
          const resolvedPath = compiler.transformModulePath(
            compiler.config.config.root,
            p
          );
          return relative(compiler.config.config.root, resolvedPath);
        })
        .join(', ');
      checkClearScreen(compiler.config.config);
      devSeverContext.logger.info(`Lazy compiling ${bold(cyan(pathsStr))}`);
      const start = Date.now();
      // sync update when node is true
      let result;
      try {
        // sync regenerate resources
        result = await compiler.update(paths, true, false, false);
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

        const returnObj = `{
          "dynamicResourcesMap": ${JSON.stringify(dynamicResourcesMap)}
        }`;
        const code = !ctx.query.node
          ? `export default ${returnObj}`
          : returnObj;
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
