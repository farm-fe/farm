import path from 'node:path';
import fse from 'fs-extra';

import merge from '../utils/merge.js';
import {
  DEFAULT_DEV_SERVER_OPTIONS,
  DEFAULT_HMR_OPTIONS
} from './constants.js';
import { NormalizedServerConfig, UserConfig } from './types.js';

function tryHttpsAsFileRead(value: unknown): string | Buffer | unknown {
  if (typeof value === 'string') {
    try {
      const resolvedPath = path.resolve(value);
      const stats = fse.statSync(resolvedPath);

      if (stats.isFile()) {
        return fse.readFileSync(resolvedPath);
      }
    } catch {}
  }

  return Buffer.isBuffer(value) ? value : value;
}

export function normalizeDevServerConfig(
  userConfig: UserConfig | undefined
): NormalizedServerConfig {
  const serverOptions = userConfig?.server;
  const { host, port, hmr: hmrConfig, https } = serverOptions || {};
  const hmr =
    hmrConfig === false || userConfig?.mode === 'production'
      ? false
      : merge(
          {},
          DEFAULT_HMR_OPTIONS,
          {
            host: host ?? DEFAULT_DEV_SERVER_OPTIONS.host,
            port: port ?? DEFAULT_DEV_SERVER_OPTIONS.port
          },
          hmrConfig === true ? {} : hmrConfig
        );

  return merge({}, DEFAULT_DEV_SERVER_OPTIONS, serverOptions, {
    hmr,
    protocol: https ? 'https' : 'http',
    https: https
      ? {
          ...https,
          ca: tryHttpsAsFileRead(serverOptions.https.ca),
          cert: tryHttpsAsFileRead(serverOptions.https.cert),
          key: tryHttpsAsFileRead(serverOptions.https.key),
          pfx: tryHttpsAsFileRead(serverOptions.https.pfx)
        }
      : undefined,
    preview: merge(
      {},
      DEFAULT_DEV_SERVER_OPTIONS.preview,
      serverOptions?.preview || {},
      {
        distDir:
          serverOptions?.preview?.distDir ||
          userConfig?.compilation?.output?.path ||
          DEFAULT_DEV_SERVER_OPTIONS.preview.distDir
      }
    )
  }) as NormalizedServerConfig;
}
