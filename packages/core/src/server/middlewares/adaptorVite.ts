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

import type Connect from 'connect';
import type { Server } from '../index.js';

export function adaptorViteMiddleware(app: Server): Connect.NextHandleFunction {
  const { resolvedUserConfig, compiler } = app;
  return function handleAdaptorViteMiddleware(req, res, next) {
    let stripQueryAndHashUrl = stripQueryAndHash(req.url);

    const { resourceWithoutPublicPath } = normalizePathByPublicPath(
      app.publicPath,
      stripQueryAndHashUrl
    );

    // try local file system
    const localFilePath = path.join(
      compiler.config.config.root,
      resourceWithoutPublicPath
    );

    // TODO maybe we can use sirv to serve static file
    // try local file system
    if (existsSync(localFilePath) && statSync(localFilePath).isFile()) {
      const headers = resolvedUserConfig.server.headers;
      send(req, res, readFileSync(localFilePath), stripQueryAndHashUrl, {
        headers
      });
      return;
    }
    next();
  };
}
