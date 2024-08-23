import fs from 'fs';
import path from 'node:path';

import { PersistentCacheConfig } from '../types/binding.js';

export function getCacheDir(
  root: string,
  persistentCache?: boolean | PersistentCacheConfig
) {
  let cacheDir: string;

  if (typeof persistentCache === 'object' && persistentCache.cacheDir) {
    cacheDir = path.isAbsolute(cacheDir)
      ? persistentCache.cacheDir
      : path.resolve(root, cacheDir);
  } else {
    cacheDir = path.resolve(root, 'node_modules', '.farm', 'cache');
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
