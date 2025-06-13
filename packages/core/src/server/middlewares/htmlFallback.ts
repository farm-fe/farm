import { cleanUrl, removeSlash } from '../../utils/index.js';

import type Connect from 'connect';
import type { Server } from '../index.js';
import { send } from '../send.js';

export function htmlFallbackMiddleware(
  app: Server
): Connect.NextHandleFunction {
  return async function htmlFallbackMiddleware(req, res, next) {
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
    const pathname = removeSlash(decodeURIComponent(url));
    const headers = app.config.server.headers;

    if (pathname.endsWith('.html')) {
      const html = app.compiler.resource(pathname);
      if (html) {
        send(req, res, html, pathname, { headers });
        return;
      }
    } else if (pathname === '') {
      const html = app.compiler.resource('index.html');
      if (html) {
        send(req, res, html, pathname, { headers });
        return;
      }
      res.statusCode = 200;
      res.setHeader('Content-Type', 'text/html');
      res.end('');
      return;
    } else {
      const html = app.compiler.resource(pathname + '.html');
      if (html) {
        send(req, res, html, pathname, { headers });
        return;
      }
    }
    if (app.serverOptions.appType === 'spa') {
      const html = app.compiler.resource('index.html');
      if (html) {
        send(req, res, html, pathname, { headers });
      } else {
        res.statusCode = 200;
        res.setHeader('Content-Type', 'text/html');
        res.end('');
      }
      return;
    }

    next();
  };
}
