/**
 * Lazy compilation middleware. Using the same logic as HMR middleware, but
 */

import { Context } from 'koa';
import chalk from 'chalk';

import { DevServer } from '../index.js';

export function lazyCompilation(server: DevServer) {
  const compiler = server.getCompiler();

  return async (ctx: Context, next: () => Promise<any>) => {
    await next();

    if (ctx.path === '/__lazy_compile') {
      const paths = (ctx.query.paths as string).split(',');

      server.logger.info(`Lazy compiling ${chalk.cyan(paths.join(', '))}...`);
      const start = Date.now();
      const result = await compiler.update(paths);
      server.logger.info(
        `Lazy compilation done for ${chalk.cyan(
          paths.join(', ')
        )} in ${chalk.green(`${Date.now() - start}ms`)}.`
      );

      if (result) {
        const code = `export default {
          modules: ${result.modules.trim().slice(0, -1)},
        }`;
        ctx.type = 'application/javascript';
        ctx.body = code;
      } else {
        throw new Error(`Lazy compilation result not found for paths ${paths}`);
      }
    }
  };
}
