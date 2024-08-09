/**
 * when use vite-plugin-vue some assets resource not compiled in dev mode
 * so we need to invalidate vite handler to recompile
 * and automatically res.body to resolve this asset resource e.g: img
 * if resource is image or font, try it in local file system to be compatible with vue
 */

import { existsSync, readFileSync, statSync } from 'node:fs';
import path from 'node:path';
import { stripQueryAndHash } from '../../utils/path.js';
import { normalizePathByPublicPath } from '../publicDir.js';
import { send } from '../send.js';

export function adaptorViteMiddleware(app: any) {
  return function handleAdaptorViteMiddleware(
    req: any,
    res: any,
    next: () => void
  ) {
    let stripQueryAndHashUrl = stripQueryAndHash(req.url);
    const { resourceWithoutPublicPath } = normalizePathByPublicPath(
      app.publicPath,
      stripQueryAndHashUrl
    );

    // try local file system
    const localFilePath = path.join(
      app.compiler.config.config.root,
      resourceWithoutPublicPath
    );

    // try local file system
    if (existsSync(localFilePath) && statSync(localFilePath).isFile()) {
      const headers = app.resolvedUserConfig.server.headers;
      send(req, res, readFileSync(localFilePath), stripQueryAndHashUrl, {
        headers
      });
      return;
    }
    next();
  };
}
