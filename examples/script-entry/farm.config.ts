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
      targetEnv: 'library',
      format: 'esm'
    },
    presetEnv: false,
    // mode: 'development',
    external: [
      ...builtinModules.map((m) => `^node:${m}$`),
      ...builtinModules.map((m) => `^${m}$`)
    ],
    runtime: {
      isolate: true
    },
    minify: false,
    mode: 'development',
    partialBundling: {
      enforceResources: [
        {
          name: 'xxx',
          test: ['.+']
        }
      ]
    },
    persistentCache: false,
    lazyCompilation: false
  },
  server: {
    hmr: false
  }
});
