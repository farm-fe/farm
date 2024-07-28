import sirv from 'sirv';
import { ResolvedUserConfig } from '../../config/types.js';
import { normalizePath } from '../../utils/share.js';
import {
  cleanUrl,
  isImportRequest,
  knownJavascriptExtensionRE
} from '../../utils/url.js';

export function publicMiddleware(
  config: ResolvedUserConfig,
  publicFiles?: Set<string>
) {
  const dir = config.publicDir;
  const headers = config.server.headers;
  const serve = sirv(dir, {
    dev: true,
    etag: true,
    extensions: [],
    setHeaders: (res, path) => {
      if (knownJavascriptExtensionRE.test(path)) {
        res.setHeader('Content-Type', 'text/javascript');
      }
      if (headers) {
        for (const name in headers) {
          res.setHeader(name, headers[name]!);
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
    if (
      (publicFiles && !publicFiles.has(toFilePath(req.url!))) ||
      isImportRequest(req.url!)
    ) {
      return next();
    }
    serve(req, res, next);
  };
}
