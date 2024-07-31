import sirv from 'sirv';

import { colors } from '../../utils/color.js';
import { Logger } from '../../utils/logger.js';
import { removeHashFromPath, withTrailingSlash } from '../../utils/path.js';
import { normalizePath } from '../../utils/share.js';
import {
  cleanUrl,
  isImportRequest,
  knownJavascriptExtensionRE,
  removeImportQuery,
  urlRE
} from '../../utils/url.js';

import type { ResolvedUserConfig } from '../../config/types.js';

function warnAboutPublicDir(url: string, publicPath: string) {
  let warning: string;
  if (isImportRequest(url)) {
    const rawUrl = removeImportQuery(url);
    if (urlRE.test(url)) {
      warning =
        `Assets in the public directory are directly accessible at the root path.\n` +
        `Use ${colors.brandColor(
          rawUrl.replace(publicPath, '/')
        )} instead of the explicit ${colors.brandColor(rawUrl)}.`;
    } else {
      warning =
        'Assets in the public directory should not be imported directly in JavaScript.\n' +
        `To import an asset, place it inside the src directory. Use ${colors.brandColor(
          rawUrl.replace(publicPath, '/src/')
        )} instead of ${colors.cyan(rawUrl)}.\n` +
        `For referencing the asset's path, use ${colors.brandColor(
          rawUrl.replace(publicPath, '/')
        )}.`;
    }
  } else {
    warning =
      `Public directory files are accessible directly at the root path.\n` +
      `Use ${colors.brandColor(
        url.replace(publicPath, '/')
      )} directly, rather than ${colors.brandColor(`${publicPath}${url}`)}.`;
  }

  return warning;
}

export function publicMiddleware(
  logger: Logger,
  config: ResolvedUserConfig,
  publicFiles?: Set<string>
) {
  const { publicDir, root } = config;
  const publicPath = `${publicDir.slice(root.length)}`;
  const headers = config.server.headers;
  const serve = sirv(publicDir, {
    dev: true,
    etag: true,
    extensions: [],
    setHeaders: (res, path) => {
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
    const url = removeHashFromPath(req.url);
    const filePath = toFilePath(url);

    // If it is not equal, it means that it is recognized as a module
    if (
      publicDir.startsWith(withTrailingSlash(root)) &&
      publicFiles.has(url) &&
      req.url !== url
    ) {
      const publicDirWarning = warnAboutPublicDir(url, publicPath);
      if (publicDirWarning) {
        logger.warn(publicDirWarning);
      }
    }

    if (publicFiles && !publicFiles.has(filePath)) {
      return next();
    }

    serve(req, res, next);
  };
}
