import { cleanUrl, withTrailingSlash } from '../../utils/index.js';

import type Connect from 'connect';
import type { Server } from '../index.js';

export function publicPathMiddleware(app: Server): Connect.NextHandleFunction {
  const { publicPath, serverOptions } = app;
  return function handlePublicPathMiddleware(req, res, next) {
    // auto redirect to public path
    const url = cleanUrl(req.url);

    if (url.startsWith(publicPath)) {
      req.url = stripBase(url, publicPath);
      return next();
    }

    if (serverOptions.middlewareMode) {
      return next();
    }

    if (url === '/' || url === '/index.html') {
      // redirect root visit to based url with search and hash
      res.writeHead(302, {
        Location: `${publicPath}${url.slice(url.length)}`
      });
      res.end();
      return;
    }

    const redirectPath =
      withTrailingSlash(url) !== publicPath
        ? joinUrlSegments(publicPath, url)
        : publicPath;

    if (req.headers.accept?.includes('text/html')) {
      res.writeHead(404, {
        'Content-Type': 'text/html'
      });
      res.end(
        `The server is configured with a public base URL of ${publicPath} - ` +
          `did you mean to visit <a href="${redirectPath}">${redirectPath}</a> instead?`
      );
      return;
    } else {
      // not found for resources
      res.writeHead(404, {
        'Content-Type': 'text/plain'
      });
      res.end(
        `The server is configured with a public base URL of ${publicPath} - ` +
          `did you mean to visit ${redirectPath} instead?`
      );
      return;
    }
  };
}

export function stripBase(path: string, base: string): string {
  if (path === base) {
    return '/';
  }
  const devBase = withTrailingSlash(base);
  return path.startsWith(devBase) ? path.slice(devBase.length - 1) : path;
}

export function joinUrlSegments(a: string, b: string): string {
  if (!a || !b) {
    return a || b || '';
  }
  if (a[a.length - 1] === '/') {
    a = a.substring(0, a.length - 1);
  }
  if (b[0] !== '/') {
    b = '/' + b;
  }
  return a + b;
}
