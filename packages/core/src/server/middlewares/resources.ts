/**
 * Serve resources that stored in memory. This middleware will be enabled when server.writeToDisk is false.
 */

import { extname } from 'node:path';
import { Context, Next } from 'koa';
import { Compiler } from '../../compiler/index.js';
import { DevServer } from '../index.js';
import koaStatic from 'koa-static';
import { NormalizedServerConfig } from '../../config/types.js';

export function resources(compiler: Compiler, config: NormalizedServerConfig) {
  return async (ctx: Context, next: Next) => {
    await next();

    if (ctx.method !== 'HEAD' && ctx.method !== 'GET') return;
    // the response is already handled
    if (ctx.body || ctx.status !== 404) return;

    // if compiler is compiling, wait for it to finish
    if (compiler.compiling) {
      await new Promise((resolve) => {
        compiler.onUpdateFinish(() => resolve(undefined));
      });
    }

    // Fallback to index.html if the resource is not found
    const resourcePath = ctx.path.slice(1) || 'index.html'; // remove leading slash
    const resource = compiler.resources()[resourcePath];

    // if resource is not found and spa is not disabled, find the closest index.html from resourcePath
    if (!resource && config.spa !== false) {
      const pathComps = resourcePath.split('/');

      while (pathComps.length > 0) {
        const pathStr = pathComps.join('/') + '.html';
        const resource = compiler.resources()[pathStr];

        if (resource) {
          ctx.type = '.html';
          ctx.body = resource;
          return;
        }

        pathComps.pop();
      }

      const indexHtml = compiler.resources()['index.html'];

      if (indexHtml) {
        ctx.type = '.html';
        ctx.body = indexHtml;
        return;
      }
      // cannot find index.html, return 404
      ctx.status = 404;
    } else {
      ctx.type = extname(resourcePath);
      ctx.body = resource;
    }
  };
}

export function resourcesPlugin(distance: DevServer) {
  distance._context.app.use(
    resources(distance._context.compiler, distance.config)
  );
  distance._context.app.use(koaStatic(distance.publicPath));
}
