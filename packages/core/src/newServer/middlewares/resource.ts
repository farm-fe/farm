import { Readable } from 'node:stream';
import mime from 'mime';
import { extname } from 'path/posix';
import { Compiler } from '../../compiler/index.js';
import { ResolvedUserConfig } from '../../config/types.js';
import {
  generateFileTree,
  generateFileTreeHtml,
  stripQueryAndHash
} from '../../utils/index.js';
import { cleanUrl } from '../../utils/url.js';
import { HttpServer } from '../index.js';
import { send } from '../send.js';
interface RealResourcePath {
  resourcePath: string;
  rawPath: string;
  resource: Buffer;
}

export function resourceMiddleware(app: any) {
  return async function generateResourceMiddleware(
    req: any,
    res: any,
    next: () => void
  ) {
    if (res.writableEnded) {
      return next();
    }

    const url = req.url && cleanUrl(req.url);
    const { compiler, resolvedUserConfig: config, publicPath } = app;
    // TODO resolve html but not input file html
    // htmlFallbackMiddleware appends '.html' to URLs
    // if (url?.endsWith('.html') && req.headers['sec-fetch-dest'] !== 'script') {
    // }

    if (compiler.compiling) {
      await new Promise((resolve) => {
        compiler.onUpdateFinish(() => resolve(undefined));
      });
    }

    const resourceResult: any = findResource(url, compiler, res, publicPath);

    if (resourceResult === true) {
      next();
    }

    if (resourceResult) {
      // need judge if resource is a deps node_modules set cache-control to 1 year
      const headers = config.server.headers || {};
      send(req, res, resourceResult.resource, url, { headers });
      return;
    }

    next();
  };
}

function findResource(
  paths: string,
  compiler: Compiler,
  res: any,
  publicPath: string
): true | undefined | RealResourcePath {
  // output_files
  if (paths === '_output_files') {
    const files = Object.keys(compiler.resources()).sort();
    const fileTree = generateFileTree(files);
    res.type = '.html';
    res.body = generateFileTreeHtml(fileTree);
    return true;
  }

  const { resourceWithoutPublicPath } = normalizePathByPublicPath(
    publicPath,
    paths
  );

  const resource = compiler.resource(resourceWithoutPublicPath);

  if (resource) {
    return {
      resource,
      resourcePath: resourceWithoutPublicPath,
      rawPath: paths
    };
  }
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
