import path from 'node:path/posix';

import mime from 'mime';

import { Compiler } from '../../compiler/index.js';
import {
  cleanUrl,
  generateFileTree,
  generateFileTreeHtml
} from '../../utils/index.js';
import { normalizePathByPublicPath } from '../publicDir.js';
import { send } from '../send.js';

import type Connect from 'connect';
import type { Server } from '../index.js';

type RealResourcePath = {
  resourcePath: string;
  rawPath: string;
  resource: Buffer;
};

export function resourceMiddleware(app: Server): Connect.NextHandleFunction {
  return async function generateResourceMiddleware(req, res, next) {
    if (res.writableEnded) {
      return next();
    }
    const url = cleanUrl(req.url);

    const { compiler, resolvedUserConfig: config, publicPath } = app;

    if (compiler._isInitialCompile) {
      await compiler.waitForInitialCompileFinish();
    } else {
      if (compiler.compiling) {
        await new Promise<void>((resolve) =>
          compiler.onUpdateFinish(() => resolve())
        );
      }
    }

    const resourceResult: any = findResource(req, res, compiler, publicPath);

    if (resourceResult === true) {
      return next();
    }
    // TODO if write to dist should be use sirv middleware
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

function findResource(
  req: any,
  res: any,
  compiler: Compiler,
  publicPath: string
): true | undefined | RealResourcePath {
  const url = req.url && cleanUrl(req.url);
  // output_files
  if (url === '_output_files') {
    const files = Object.keys(compiler.resources()).sort();
    const fileTree = generateFileTree(files);
    res.type = '.html';
    res.body = generateFileTreeHtml(fileTree);
    return true;
  }

  const { resourceWithoutPublicPath } = normalizePathByPublicPath(
    publicPath,
    url
  );

  const resource = compiler.resource(resourceWithoutPublicPath);

  if (resource) {
    return {
      resource,
      resourcePath: resourceWithoutPublicPath,
      rawPath: url
    };
  }
}
