import path from 'node:path';

import chokidar, { FSWatcher, WatchOptions } from 'chokidar';
import glob from 'fast-glob';

import { ResolvedUserConfig, getCacheDir } from '../index.js';

function resolveChokidarOptions(
  config: ResolvedUserConfig,
  insideChokidarOptions: WatchOptions
) {
  const { ignored = [], ...userChokidarOptions } =
    config.server?.hmr?.watchOptions ?? {};

  const cacheDir = getCacheDir(
    config.root,
    config.compilation?.persistentCache
  );

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
    ...userChokidarOptions,
    ...insideChokidarOptions
  };

  return options;
}

export function createWatcher(
  config: ResolvedUserConfig,
  files: string[],
  chokidarOptions?: WatchOptions
): FSWatcher {
  const options = resolveChokidarOptions(config, chokidarOptions);

  return chokidar.watch(files, options);
}
