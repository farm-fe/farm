/**
 * Lazy compilation middleware. Using the same logic as HMR middleware, but
 */

import { relative } from 'node:path';
import { Context } from 'koa';
import chalk from 'chalk';

import { DevServer } from '../index.js';
import type { Resource } from '@farmfe/runtime/src/resource-loader.js';

export function lazyCompilation(server: DevServer) {
  const compiler = server.getCompiler();

  return async (ctx: Context, next: () => Promise<any>) => {
    await next();

    if (ctx.path === '/__lazy_compile') {
      const paths = (ctx.query.paths as string).split(',');
      const pathsStr = paths
        .map((p) => {
          const resolvedPath = compiler.transformModulePath(
            compiler.config.config.root,
            p
          );
          return relative(compiler.config.config.root, resolvedPath);
        })
        .join(', ');
      server.logger.info(`Lazy compiling ${chalk.cyan(pathsStr)}...`);
      const start = Date.now();
      const result = await compiler.update(paths);
      server.logger.info(
        `Lazy compilation done for ${chalk.cyan(pathsStr)} in ${chalk.green(
          `${Date.now() - start}ms`
        )}.`
      );

      server.hmrEngine.callUpdates(result);

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
          modules: ${result.modules.trim().slice(0, -1)},
          dynamicResourcesMap: ${JSON.stringify(dynamicResourcesMap)}
        }`;
        ctx.type = 'application/javascript';
        ctx.body = code;
      } else {
        throw new Error(`Lazy compilation result not found for paths ${paths}`);
      }
    }
  };
}

export function lazyCompilationPlugin(distance: DevServer) {
  if (distance._context.compiler.config.config.lazyCompilation) {
    distance._context.app.use(lazyCompilation(distance));
  }
}
