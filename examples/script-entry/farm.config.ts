import { defineConfig } from '@farmfe/core';
import { builtinModules } from 'module';

/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default defineConfig({
  compilation: {
    input: {
      index: './index.ts'
    },
    output: {
      path: 'dist/esm',
      entryFilename: '[entryName].mjs',
      targetEnv: 'node',
      format: 'esm'
    },
    mode: 'development',
    external: [
      ...builtinModules.map((m) => `^node:${m}$`),
      ...builtinModules.map((m) => `^${m}$`)
    ],
    minify: false,
    persistentCache: false,
    lazyCompilation: false,
  },
  server: {
    hmr: false
  }
});
