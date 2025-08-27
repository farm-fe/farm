/**
 * when use vite-plugin-vue some assets resource not compiled in dev mode
 * so we need to invalidate vite handler to recompile
 * and automatically res.body to resolve this asset resource e.g: img
 * if resource is image or font, try it in local file system to be compatible with vue
 */

import { existsSync, statSync } from 'node:fs';
import path from 'node:path';

import {
  cleanUrl,
  fsPathFromUrl,
  isImportRequest,
  normalizePath,
  removeLeadingSlash
} from '../../utils/index.js';
import { stripQueryAndHash, withTrailingSlash } from '../../utils/path.js';

import { OutgoingHttpHeaders } from 'node:http';
import type Connect from 'connect';
import sirv, { Options } from 'sirv';
import type { Server } from '../index.js';

export function staticMiddleware(app: Server): Connect.NextHandleFunction {
  const { config, root } = app;
  const serve = sirv(
    root,
    sirvOptions({
      getHeaders: () => config.server.headers
    })
  );
  return function handleStaticMiddleware(req, res, next) {
    let stripQueryAndHashUrl = stripQueryAndHash(req.url);

    if (
      stripQueryAndHashUrl[stripQueryAndHashUrl.length - 1] === '/' ||
      path.extname(stripQueryAndHashUrl) === '.html'
    ) {
      return next();
    }

    // try local file system
    let fileUrl = path.resolve(root, removeLeadingSlash(stripQueryAndHashUrl));
    if (
      stripQueryAndHashUrl[stripQueryAndHashUrl.length - 1] === '/' &&
      fileUrl[fileUrl.length - 1] !== '/'
    ) {
      fileUrl = withTrailingSlash(fileUrl);
    }
    const filePath = fsPathFromUrl(fileUrl);

    // TODO FS.allow FS.deny server.fs.allow server.fs.deny
    if (existsSync(filePath) && statSync(filePath).isFile()) {
      serve(req, res, next);
    } else {
      next();
    }
  };
}

export function publicMiddleware(app: Server): Connect.NextHandleFunction {
  const { config: config, publicDir, publicFiles } = app;
  const serve = sirv(
    publicDir,
    sirvOptions({
      getHeaders: () => config.server.headers
    })
  );
  const toFilePath = (url: string) => {
    let filePath = cleanUrl(url);
    if (filePath.indexOf('%') !== -1) {
      try {
        filePath = decodeURI(filePath);
      } catch {}
    }
    return normalizePath(filePath);
  };

  return async function farmHandlerPublicMiddleware(
    req,
    res,
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

const knownJavascriptExtensionRE = /\.[tj]sx?$/;

export const sirvOptions = ({
  getHeaders
}: {
  getHeaders: () => OutgoingHttpHeaders | undefined;
}): Options => {
  return {
    dev: true,
    etag: true,
    extensions: [],
    setHeaders(res, pathname) {
      // Matches js, jsx, ts, tsx.
      // The reason this is done, is that the .ts file extension is reserved
      // for the MIME type video/mp2t. In almost all cases, we can expect
      // these files to be TypeScript files, and for Vite to serve them with
      // this Content-Type.
      if (knownJavascriptExtensionRE.test(pathname)) {
        res.setHeader('Content-Type', 'text/javascript');
      }
      const headers = getHeaders();
      if (headers) {
        for (const name in headers) {
          res.setHeader(name, headers[name]!);
        }
      }
    }
  };
};
