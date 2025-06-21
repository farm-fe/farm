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
  getDynamicResources,
  green
} from '../../index.js';
import { Server } from '../index.js';

import { existsSync } from 'node:fs';
import { getValidPublicPath } from '../../config/normalize-config/normalize-output.js';
import { logError } from '../error.js';

export function lazyCompilation(devSeverContext: Server): Middleware {
  const compiler = devSeverContext.getCompiler();

  if (!compiler.config.config?.lazyCompilation) {
    return;
  }

  return async (ctx: Context, next: Next) => {
    const publicPath = getValidPublicPath(
      compiler.config.config?.output?.publicPath
    );

    if (ctx.path === `${publicPath || '/'}__lazy_compile`) {
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

      if (ctx.query.node || devSeverContext.config.writeToDisk) {
        compiler.writeResourcesToDisk();
      }

      devSeverContext.logger.info(
        `${bold(green(`✓`))} Lazy compilation done(${bold(
          cyan(pathsStr)
        )}) in ${bold(green(`${Date.now() - start}ms`))}.`
      );

      if (result) {
        const { dynamicResources, dynamicModuleResourcesMap } =
          getDynamicResources(result.dynamicResourcesMap);

        const returnObj = `{
          "dynamicResources": ${JSON.stringify(dynamicResources)},
          "dynamicModuleResourcesMap": ${JSON.stringify(
            dynamicModuleResourcesMap
          )}
        }`;
        const code = !ctx.query.node
          ? `export default ${returnObj}`
          : returnObj;
        ctx.type = !ctx.query.node
          ? 'application/javascript'
          : 'application/json';
        ctx.body = code;
        // enable cors
        ctx.set('Access-Control-Allow-Origin', '*');
        ctx.set('Access-Control-Allow-Methods', '*');
        ctx.set('Access-Control-Allow-Headers', '*');
      } else {
        throw new Error(`Lazy compilation result not found for paths ${paths}`);
      }
    } else {
      await next();
    }
  };
}
