import { Readable } from 'node:stream';
import mime from 'mime';
import path, { extname } from 'path/posix';
import { Compiler } from '../../compiler/index.js';
import { ResolvedUserConfig } from '../../config/types.js';
import {
  generateFileTree,
  generateFileTreeHtml,
  stripQueryAndHash
} from '../../utils/index.js';
import { cleanUrl } from '../../utils/url.js';
import { HttpServer } from '../index.js';
import { normalizePathByPublicPath } from '../publicDir.js';
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
    const url = cleanUrl(req.url);

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

    const resourceResult: any = findResource(req, res, compiler, publicPath);

    if (resourceResult === true) {
      next();
    }
    // TODO if write to dist should be use sirv middleware
    if (resourceResult) {
      // need judge if resource is a deps node_modules set cache-control to 1 year
      const headers = config.server.headers;
      send(req, res, resourceResult.resource, url, { headers });
      return;
    }

    // publicPath
    // 处理找不到资源的情况

    const { resourceWithoutPublicPath } = normalizePathByPublicPath(
      publicPath,
      url
    );

    const extension = path.extname(resourceWithoutPublicPath).toLowerCase();
    const mimeType = mime.getType(extension) || 'application/octet-stream';
    const isHtmlRequest =
      mimeType === 'text/html' ||
      (extension === '' && req.headers.accept?.includes('text/html'));

    // TODO 这块是 vite vue 插件处理 图片的时候找不到 资源应该考虑在上面的 vite 插件里做
    if (!isHtmlRequest) {
      // 对于非 HTML 请求，尝试在根目录查找资源
      const rootResource = compiler.resource(
        path.basename(resourceWithoutPublicPath)
      );

      if (rootResource) {
        send(req, res, rootResource, url, {
          headers: config.server.headers
        });
        return;
      }
      // 如果在根目录也找不到，返回 404
      res.statusCode = 404;
      res.end('Not found');
      return;
    }

    if (config.spa !== false) {
      let indexHtml = compiler.resources()['index.html'];

      if (indexHtml) {
        res.setHeader('Content-Type', 'text/html');
        res.end(indexHtml);
        return;
      }
    }

    // 如果找不到任何匹配的资源，返回 404
    res.statusCode = 404;
    res.end('Not found');
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
