import fs from 'fs';
import path from 'node:path';

import { PersistentCacheConfig } from '../types/binding';

export function getCacheDir(
  root: string,
  persistentCache?: boolean | PersistentCacheConfig
) {
  let cacheDir = path.resolve(root, 'node_modules', '.farm', 'cache');

  if (typeof persistentCache === 'object' && persistentCache.cacheDir) {
    cacheDir = path.isAbsolute(persistentCache.cacheDir)
      ? persistentCache.cacheDir
      : path.resolve(root, persistentCache.cacheDir);
  }

  return cacheDir;
}

export async function isCacheDirExists(dir: string): Promise<boolean> {
  try {
    const hasCacheDir = fs.readdirSync(dir, { withFileTypes: true });

    return !!(hasCacheDir && hasCacheDir.length);
  } catch (_) {
    return false;
  }
}
