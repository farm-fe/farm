/**
 * Serve resources that stored in memory. This middleware will be enabled when server.writeToDisk is false.
 */

import { ReadStream, existsSync, readFileSync, statSync } from 'node:fs';
import path, { extname } from 'node:path';
import { Context, Middleware, Next } from 'koa';
import koaStatic from 'koa-static';
import { Compiler } from '../../compiler/index.js';
import {
  generateFileTree,
  generateFileTreeHtml,
  stripQueryAndHash
} from '../../utils/index.js';
import { Server } from '../index.js';

interface RealResourcePath {
  resourcePath: string;
  rawPath: string;
  resource: Buffer;
}

function normalizePathByPublicPath(publicPath: string, resourcePath: string) {
  const base = publicPath.match(/^https?:\/\//) ? '' : publicPath;
  let resourceWithoutPublicPath = resourcePath;

  if (base && resourcePath.startsWith(base)) {
    resourcePath = resourcePath.replace(new RegExp(`([^/]+)${base}`), '$1/');
    resourceWithoutPublicPath = resourcePath.slice(base.length);
  }

  return { resourceWithoutPublicPath, fullPath: resourcePath };
}

function outputFilesMiddleware(compiler: Compiler): Middleware {
  return async (ctx: Context, next: Next) => {
    if (ctx.path === '/_output_files') {
      const files = Object.keys(compiler.resources()).sort();
      const fileTree = generateFileTree(files);
      ctx.type = '.html';
      ctx.body = generateFileTreeHtml(fileTree);
    } else {
      await next();
    }
  };
}

function findResource(
  paths: string[],
  compiler: Compiler,
  publicPath: string
): true | undefined | RealResourcePath {
  for (const resourcePath of new Set(paths)) {
    const { resourceWithoutPublicPath } = normalizePathByPublicPath(
      publicPath,
      resourcePath
    );

    const resource = compiler.resource(resourceWithoutPublicPath);

    if (resource) {
      return {
        resource,
        resourcePath: resourceWithoutPublicPath,
        rawPath: resourcePath
      };
    }
  }
}

export function resourcesMiddleware(compiler: Compiler, serverContext: Server) {
  return async (ctx: Context, next: Next) => {
    await next();
    if (ctx.method !== 'HEAD' && ctx.method !== 'GET') return;
    const hasHtmlPathWithPublicDir = path.resolve(
      serverContext.publicDir,
      'index.html'
    );

    let isSkipPublicHtml;
    if (ctx.body instanceof ReadStream) {
      const readStream = ctx.body;
      isSkipPublicHtml = readStream.path === hasHtmlPathWithPublicDir;
    }

    // the response is already handled
    if ((ctx.body || ctx.status !== 404) && !isSkipPublicHtml) return;

    const { config, publicPath } = serverContext;
    // if compiler is compiling, wait for it to finish
    if (compiler.compiling) {
      await new Promise((resolve) => {
        compiler.onUpdateFinish(() => resolve(undefined));
      });
    }
    // Fallback to index.html if the resource is not found
    const url = ctx.url?.slice(1) || 'index.html'; // remove leading slash

    let stripQueryAndHashUrl = stripQueryAndHash(url);
    const resourceResult = findResource(
      [url, stripQueryAndHashUrl],
      compiler,
      publicPath
    );

    if (resourceResult === true) {
      return;
    }

    if (resourceResult) {
      ctx.type = extname(ctx?.path?.slice?.(1) || 'index.html');
      ctx.body = resourceResult.resource;
      return;
    }

    const { fullPath, resourceWithoutPublicPath } = normalizePathByPublicPath(
      publicPath,
      stripQueryAndHashUrl
    );

    // if resource is image or font, try it in local file system to be compatible with vue
    {
      // try local file system
      const absPath = path.join(
        compiler.config.config.root,
        resourceWithoutPublicPath
      );
      // const mimeStr = mime.lookup(absPath);

      if (
        existsSync(absPath) &&
        statSync(absPath).isFile()
        // mimeStr &&
        // (mimeStr.startsWith('image') || mimeStr.startsWith('font'))
      ) {
        ctx.type = extname(fullPath);
        ctx.body = readFileSync(absPath);
        return;
      }
    }

    //   // try local file system with publicDir
    //   const absPathPublicDir = path.resolve(
    //     compiler.config.config.root,
    //     compiler.config.config.assets.publicDir,
    //     resourceWithoutPublicPath
    //   );

    //   if (existsSync(absPathPublicDir) && statSync(absPathPublicDir).isFile()) {
    //     ctx.type = extname(fullPath);
    //     ctx.body = readFileSync(absPathPublicDir);
    //     return;
    //   }
    // }

    // // if resource is not found and spa is not disabled, find the closest index.html from resourcePath
    // {
    //   // if request mime is not html, return 404
    //   if (!ctx.accepts('html')) {
    //     ctx.status = 404;
    //   } else if (config.spa !== false) {
    //     const pathComps = resourceWithoutPublicPath.split('/');

    //     while (pathComps.length > 0) {
    //       const pathStr = pathComps.join('/') + '.html';
    //       const resource = compiler.resources()[pathStr];

    //       if (resource) {
    //         ctx.type = '.html';
    //         ctx.body = resource;
    //         return;
    //       }

    //       pathComps.pop();
    //     }

    //     const indexHtml = compiler.resources()['index.html'];

    //     if (indexHtml) {
    //       ctx.type = '.html';
    //       ctx.body = indexHtml;
    //       return;
    //     }
    //   } else {
    //     // cannot find index.html, return 404
    //     ctx.status = 404;
    //   }
    // }
  };
}

export function resources(devSeverContext: Server): Middleware | Middleware[] {
  const middlewares = [outputFilesMiddleware(devSeverContext.getCompiler())];
  if (!devSeverContext.config.writeToDisk) {
    middlewares.push(
      resourcesMiddleware(devSeverContext.getCompiler(), devSeverContext)
    );
  } else {
    middlewares.push(
      koaStatic(devSeverContext.getCompiler().config.config.output.path, {
        extensions: ['html']
      })
    );
  }

  middlewares.push(koaStatic(devSeverContext.publicDir));
  return middlewares;
}
