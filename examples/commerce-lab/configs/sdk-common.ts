import path from 'node:path';
import type { UserConfig } from '@farmfe/core';
export function sdkCommon(group: 'js' | 'rust') {
  const cacheId = process.env.COMMERCE_SDK_CACHE_ID;
  if (cacheId && !/^[a-zA-Z0-9_-]+$/.test(cacheId)) throw new Error('Invalid SDK cache ID');
  const format = process.env.COMMERCE_SDK_FORMAT ?? 'esm';
  if (format !== 'esm' && format !== 'cjs') throw new Error('Invalid SDK module format');
  return {
    root: path.resolve('packages/sdk'),
    compilation: {
      input: { index: './src/index.ts' },
      output: { path: path.resolve(`dist/sdk-${group}${format === 'cjs' ? '-cjs' : ''}`), targetEnv: 'library', format, entryFilename: `[entryName].${format === 'cjs' ? 'cjs' : 'js'}` },
      persistentCache: cacheId ? { cacheDir: path.resolve('.cache', `sdk-${group}`, cacheId, format) } : false,
      minify: false, sourcemap: false, presetEnv: false,
    },
  } satisfies UserConfig;
}
