import { defineConfig } from 'farm';

/**
 * @type {import('farm').UserConfig}
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
