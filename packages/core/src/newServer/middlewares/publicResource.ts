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

// TODO: if url endsWith importQueryRE we need can check if it is a module then serve or not
// function warnAboutPublicDir(url: string, publicPath: string) {
//   let warning: string;
//   if (isImportRequest(url)) {
//     const rawUrl = removeImportQuery(url);
//     if (urlRE.test(url)) {
//       warning =
//         `Assets in the public directory are directly accessible at the root path.\n` +
//         `Use ${colors.brandColor(
//           rawUrl.replace(publicPath, "/")
//         )} instead of the explicit ${colors.brandColor(rawUrl)}.`;
//     } else {
//       warning =
//         "Assets in the public directory should not be imported directly in JavaScript.\n" +
//         `To import an asset, place it inside the src directory. Use ${colors.brandColor(
//           rawUrl.replace(publicPath, "/src/")
//         )} instead of ${colors.cyan(rawUrl)}.\n` +
//         `For referencing the asset's path, use ${colors.brandColor(
//           rawUrl.replace(publicPath, "/")
//         )}.`;
//     }
//   } else {
//     warning =
//       `Public directory files are accessible directly at the root path.\n` +
//       `Use ${colors.brandColor(
//         url.replace(publicPath, "/")
//       )} directly, rather than ${colors.brandColor(`${publicPath}${url}`)}.`;
//   }

//   return warning;
// }

export function publicMiddleware(app: any) {
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

  // return async function farmHandlerPublicMiddleware(
  //   req: any,
  //   res: any,
  //   next: () => void
  // ) {
  //   if (publicFiles && !publicFiles.has(toFilePath(req.url))) {
  //     return next();
  //   }

  //   serve(req, res, next);
  // };
  return async function farmHandlerPublicMiddleware(
    req: any,
    res: any,
    next: () => void
  ) {
    const originalUrl = req.url;
    const filePath = toFilePath(originalUrl);

    console.log('Original URL:', originalUrl);
    console.log('File path:', filePath);
    console.log('Public path:', publicPath);

    // 移除 URL 开头的 publicPath
    const urlWithoutPublicPath = filePath.startsWith('/' + publicPath)
      ? filePath.slice(publicPath.length + 1)
      : filePath;

    console.log('URL without public path:', urlWithoutPublicPath);

    // 检查文件是否在 publicFiles 中
    if (publicFiles && publicFiles.has('/' + urlWithoutPublicPath)) {
      // 如果文件在 publicFiles 中，修改 req.url
      req.url = '/' + urlWithoutPublicPath;
      console.log('Modified URL:', req.url);

      // 使用 sirv 处理请求
      serve(req, res, next);
    } else {
      // 如果文件不在 publicFiles 中，直接调用 next()
      console.log('File not found in publicFiles');
      next();
    }
  };
}
