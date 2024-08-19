import fs from 'fs';
import path from 'node:path';

import { PersistentCacheConfig } from '../types/binding.js';

export function getCacheDir(
  root: string,
  persistentCache?: boolean | PersistentCacheConfig
) {
  let cacheDir = path.resolve(root, 'node_modules', '.farm', 'cache');

  if (typeof persistentCache === 'object' && persistentCache.cacheDir) {
    cacheDir = persistentCache.cacheDir;

    if (!path.isAbsolute(cacheDir)) {
      cacheDir = path.resolve(root, cacheDir);
    }
  }

  return cacheDir;
}

export async function isCacheDirExists(dir: string): Promise<boolean> {
  try {
    const hasCacheDir = fs.readdirSync(dir, { withFileTypes: true });

    return !!(hasCacheDir && hasCacheDir.length);
  } catch (e) {
    return false;
  }
}
