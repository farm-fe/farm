import { withTrailingSlash } from '../../utils/index.js';

import type Connect from 'connect';
import { Server as DevServer } from '../index.js';
import type { PreviewServer } from '../preview.js';

export function publicPathMiddleware(
  app: DevServer | PreviewServer,
  middlewareMode: boolean
): Connect.NextHandleFunction {
  return function handlePublicPathMiddleware(req, res, next) {
    const publicPath = app.publicPath;
    // auto redirect to public path
    const url = req.url;

    if (url.startsWith(publicPath)) {
      req.url = stripBase(url, publicPath);
      return next();
    }

    if (middlewareMode) {
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
