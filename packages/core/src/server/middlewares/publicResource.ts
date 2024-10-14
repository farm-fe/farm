import sirv from 'sirv';

import {
  cleanUrl,
  knownJavascriptExtensionRE,
  normalizePath
} from '../../utils/index.js';

import type Connect from 'connect';
import type { Server } from '../index.js';

// TODO: if url endsWith importQueryRE we need can check if it is a module then serve or not

export function publicMiddleware(app: Server): Connect.NextHandleFunction {
  const {
    resolvedUserConfig: config,
    publicDir,
    publicFiles,
    publicPath
  } = app;

  const headers = config.server.headers;
  const serve = sirv(publicDir, {
    etag: true,
    setHeaders: (res, path) => {
      // res.setHeader("Cache-Control", "public,max-age=31536000,immutable");
      if (knownJavascriptExtensionRE.test(path)) {
        res.setHeader('Content-Type', 'text/javascript');
      }
      if (headers) {
        for (const name in headers) {
          res.setHeader(name, headers[name]);
        }
      }
    }
  });
  const toFilePath = (url: string) => {
    let filePath = cleanUrl(url);
    if (filePath.indexOf('%') !== -1) {
      try {
        filePath = decodeURI(filePath);
      } catch (err) {
        // ignore
      }
    }
    return normalizePath(filePath);
  };

  return async function farmHandlerPublicMiddleware(
    req: any,
    res: any,
    next: () => void
  ) {
    const originalUrl = req.url;
    const filePath = toFilePath(originalUrl);
    // TODO public 缓存问题
    // 移除 URL 开头的 publicPath
    const urlWithoutPublicPath = filePath.startsWith('/' + publicPath)
      ? filePath.slice(publicPath.length + 1)
      : filePath;

    // 检查文件是否在 publicFiles 中或者在 public 目录中
    if (
      (publicFiles && publicFiles.has(urlWithoutPublicPath)) ||
      (publicDir && serve(req, res, () => {}))
    ) {
      req.url = urlWithoutPublicPath;

      return serve(req, res, next);
    }

    next();
  };
}
