import mime from 'mime';
import { extname } from 'path/posix';
import { Compiler } from '../../compiler/index.js';
import {
  generateFileTree,
  generateFileTreeHtml,
  stripQueryAndHash
} from '../../utils/index.js';
import { cleanUrl } from '../../utils/url.js';
import { HttpServer } from '../index.js';
interface RealResourcePath {
  resourcePath: string;
  rawPath: string;
  resource: Buffer;
}

export function resourceMiddleware(
  server: HttpServer,
  compiler: Compiler,
  publicPath: string
) {
  return async function generateResourceMiddleware(
    req: any,
    res: any,
    next: () => void
  ) {
    if (res.writableEnded) {
      return next();
    }

    // const url = req.url && cleanUrl(req.url);
    const url = req.url?.slice(1) || 'index.html';

    if (compiler.compiling) {
      await new Promise((resolve) => {
        compiler.onUpdateFinish(() => resolve(undefined));
      });
    }

    let stripQueryAndHashUrl = stripQueryAndHash(url);
    const resourceResult: any = findResource(
      [url, stripQueryAndHashUrl],
      compiler,
      res,
      publicPath
    );

    if (resourceResult === true) {
      next();
    }

    if (resourceResult) {
      res.setHeader('Content-Type', mime.getType(extname(url || 'index.html')));
      res.end(resourceResult.resource);
      return;
    }

    next();
  };
}

function findResource(
  paths: string[],
  compiler: Compiler,
  res: any,
  publicPath: string
): true | undefined | RealResourcePath {
  for (const resourcePath of new Set(paths)) {
    // output_files
    if (resourcePath === '_output_files') {
      const files = Object.keys(compiler.resources()).sort();
      const fileTree = generateFileTree(files);
      res.type = '.html';
      res.body = generateFileTreeHtml(fileTree);
      return true;
    }

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

function normalizePathByPublicPath(publicPath: string, resourcePath: string) {
  const base = publicPath.match(/^https?:\/\//) ? '' : publicPath;
  let resourceWithoutPublicPath = resourcePath;

  if (base && resourcePath.startsWith(base)) {
    resourcePath = resourcePath.replace(new RegExp(`([^/]+)${base}`), '$1/');
    resourceWithoutPublicPath = resourcePath.slice(base.length);
  }

  return { resourceWithoutPublicPath, fullPath: resourcePath };
}
