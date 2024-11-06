import { defineConfig } from '@farmfe/core';
import { builtinModules } from 'module';

/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default defineConfig({
  compilation: {
    // multiple bundle recommend config
    treeShaking: false,
    partialBundling: {
      targetConcurrentRequests: Number.MAX_SAFE_INTEGER,
      targetMinSize: 1,
    },

    input: {
      index: './index.ts'
    },
    output: {
      path: 'dist/esm',
      entryFilename: '[entryName].mjs',
      targetEnv: 'library-node',
      format: 'esm'
    },
    presetEnv: false,
    // mode: 'development',
    external: [
      ...builtinModules.map((m) => `^node:${m}$`),
      ...builtinModules.map((m) => `^${m}$`)
    ],
    minify: false,
    mode: 'development',
    persistentCache: false,
    lazyCompilation: false
  },
  server: {
    hmr: false
  }
});
