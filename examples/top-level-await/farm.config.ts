import { defineConfig } from '@farmfe/core';

/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default defineConfig({
  compilation: {
    persistentCache: false,
    sourcemap: false,
  }
  // compilation: {
  //   input: {
  //     index: './src/index.js'
  //   },
  //   output: {
  //     path: 'dist/esm',
  //     entryFilename: '[entryName].mjs',
  //     targetEnv: 'node',
  //     format: 'esm'
  //   },
  //   minify: false,
  //   persistentCache: false,
  //   lazyCompilation: false,
  // },
  // server: {
  //   hmr: false
  // }
});
