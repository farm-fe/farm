import path from 'node:path';

import { cleanUrl, commonFsUtils } from '../../utils/index.js';

import type Connect from 'connect';
import { ResolvedUserConfig } from '../../config/types.js';
import type { Server } from '../index.js';

export function htmlFallbackMiddleware(app: {
  resolvedUserConfig: ResolvedUserConfig;
}): Connect.NextHandleFunction {
  return async function htmlFallbackMiddleware(req, _res, next) {
    if (
      // Only accept GET or HEAD
      (req.method !== 'GET' && req.method !== 'HEAD') ||
      // Exclude default favicon requests
      req.url === '/favicon.ico' ||
      // Require Accept: text/html or */*
      !(
        req.headers.accept === undefined || // equivalent to `Accept: */*`
        req.headers.accept === '' || // equivalent to `Accept: */*`
        req.headers.accept.includes('text/html') ||
        req.headers.accept.includes('*/*')
      )
    ) {
      return next();
    }
    const url = cleanUrl(req.url);
    const pathname = decodeURIComponent(url);
    const { resolvedUserConfig: config } = app;
    if (pathname.endsWith('.html')) {
      const filePath = path.join(config.root, pathname);
      if (commonFsUtils.existsSync(filePath)) {
        req.url = url;
        return next();
      }
    } else if (pathname[pathname.length - 1] === '/') {
      const filePath = path.join(config.root, pathname, 'index.html');
      if (commonFsUtils.existsSync(filePath)) {
        const newUrl = url + 'index.html';
        req.url = newUrl;
        return next();
      }
    } else {
      //TODO mpa not compatible 如果是纯 html 的结果 html 需要可能判断一下 mpa 适配
      const filePath = path.join(config.root, pathname + '.html');
      if (commonFsUtils.existsSync(filePath)) {
        const newUrl = url + '.html';
        req.url = newUrl;
        return next();
      }
    }

    // TODO htmlFallBack when spa
    // if (config.appType === 'spa') {
    // req.url = '/index.html';
    // }
    next();
  };
}
