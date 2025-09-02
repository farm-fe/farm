import path from 'node:path/posix';

import mime from 'mime';
import sirv from 'sirv';

import { Compiler } from '../../compiler/index.js';
import {
  cleanUrl,
  generateFileTree,
  generateFileTreeHtml
} from '../../utils/index.js';
import { normalizePathByPublicPath } from '../publicDir.js';
import { send } from '../send.js';
import { sirvOptions } from './static.js';

import type { IncomingMessage } from 'http';
import type Connect from 'connect';
import type { Server } from '../index.js';

type RealResourcePath = {
  resourcePath: string;
  rawPath: string;
  resource: Buffer;
};

async function waitForCompilerReady(server: Server) {
  if (!server.compiler) {
    return new Promise<void>((resolve) => {
      const timer = setInterval(() => {
        if (server.compiler) {
          clearInterval(timer);
          resolve();
        }
      }, 100);
    });
  }
}

export function resourceMiddleware(app: Server): Connect.NextHandleFunction {
  return async function generateResourceMiddleware(req, res, next) {
    if (res.writableEnded) {
      return next();
    }
    const url = cleanUrl(req.url);
    const { compiler, config, publicPath } = app;

    await waitForCompilerReady(app);

    if (compiler.compiling) {
      await compiler.waitForCompileFinish();
    } else {
      if (compiler.compiling) {
        await new Promise<void>((resolve) =>
          compiler.onUpdateFinish(() => resolve())
        );
      }
    }

    const resourceResult = findResource(req, compiler, publicPath);

    if (resourceResult === true) {
      return next();
    }

    if (resourceResult) {
      // need judge if resource is a deps node_modules set cache-control to 1 year
      const headers = config.server.headers;
      send(req, res, resourceResult.resource, url, { headers });
      return;
    }

    // publicPath
    const { resourceWithoutPublicPath } = normalizePathByPublicPath(
      publicPath,
      url
    );

    const extension = path.extname(resourceWithoutPublicPath).toLowerCase();
    const mimeType = mime.getType(extension) || 'application/octet-stream';

    const isHtmlRequest =
      mimeType === 'text/html' ||
      (!extension && req.headers.accept?.includes('text/html'));

    if (!isHtmlRequest) {
      const rootResource = compiler.resource(
        path.basename(resourceWithoutPublicPath)
      );
      if (rootResource) {
        send(req, res, rootResource, url, {
          headers: config.server.headers
        });
        return;
      }
      res.statusCode = 404;
      res.end('Not found');
      return;
    }

    next();
  };
}

export function resourceDiskMiddleware(
  app: Server
): Connect.NextHandleFunction {
  return async function generateResourceDiskMiddleware(req, res, next) {
    if (res.writableEnded) {
      return next();
    }
    const { config, compiler } = app;
    const root = path.join(
      compiler.config.root,
      config.compilation.output.path
    );

    const serve = sirv(
      root,
      sirvOptions({
        getHeaders: () => config.server.headers
      })
    );
    serve(req, res, next);
  };
}

export function findResource(
  req: IncomingMessage,
  compiler: Compiler,
  publicPath: string
): true | undefined | RealResourcePath {
  const { resourceWithoutPublicPath } = normalizePathByPublicPath(
    publicPath,
    req.url
  );
  const normalizedPath = resourceWithoutPublicPath.startsWith('/')
    ? resourceWithoutPublicPath.slice(1)
    : resourceWithoutPublicPath;

  const resource = compiler.resource(normalizedPath);

  if (resource) {
    return {
      resource,
      resourcePath: resourceWithoutPublicPath,
      rawPath: req.url
    };
  }
}

export function outputFilesMiddleware(app: Server): Connect.NextHandleFunction {
  return function handleOutputFiles(req, res, next) {
    if (res.writableEnded) {
      return next();
    }

    if (req.url !== '/_output_files') {
      return next();
    }

    try {
      const { compiler } = app;
      const files = Object.keys(compiler.resources()).sort();

      const fileTree = generateFileTree(files);

      res.setHeader('Content-Type', 'text/html');
      const html = generateFileTreeHtml(fileTree);
      res.write(html);
      res.end();
    } catch (error) {
      if (!res.writableEnded) {
        next(error);
      }
    }
  };
}
