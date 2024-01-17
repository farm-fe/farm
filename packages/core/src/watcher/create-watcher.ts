import path from 'node:path';

import chokidar, { FSWatcher, WatchOptions } from 'chokidar';
import glob from 'fast-glob';

import { ResolvedUserConfig } from '../index.js';

function resolveChokidarOptions(config: ResolvedUserConfig) {
  const { ignored = [], ...otherOptions } =
    config.server?.hmr?.watchOptions ?? {};
  let cacheDir = path.resolve(config.root, 'node_modules', '.farm', 'cache');

  if (
    typeof config.compilation?.persistentCache === 'object' &&
    config.compilation.persistentCache.cacheDir
  ) {
    cacheDir = config.compilation.persistentCache.cacheDir;

    if (!path.isAbsolute(cacheDir)) {
      cacheDir = path.resolve(config.root, cacheDir);
    }
  }

  const options: WatchOptions = {
    ignored: [
      '**/.git/**',
      '**/node_modules/**',
      '**/test-results/**', // Playwright
      glob.escapePath(cacheDir) + '/**',
      glob.escapePath(
        path.resolve(config.root, config.compilation.output.path)
      ) + '/**',
      ...(Array.isArray(ignored) ? ignored : [ignored])
    ],
    ignoreInitial: true,
    ignorePermissionErrors: true,
    // for windows and macos, we need to wait for the file to be written
    awaitWriteFinish:
      process.platform === 'linux'
        ? undefined
        : {
            stabilityThreshold: 10,
            pollInterval: 10
          },
    ...otherOptions
  };

  return options;
}

export function createWatcher(
  config: ResolvedUserConfig,
  files: string[]
): FSWatcher {
  const options = resolveChokidarOptions(config);

  return chokidar.watch(files, options);
}
