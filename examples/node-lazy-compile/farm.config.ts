import { defineConfig } from '@farmfe/core';

const lazyPort = 20000 + (Math.random() * 10000 >> 0);

export default defineConfig({
  compilation: {
    input: {
      index: 'src/index.ts',
    },
    lazyCompilation: true,
    progress: false,
    persistentCache: false,
    output: {
      format: 'esm',
      targetEnv: 'node',
      entryFilename: '[entryName].mjs',
      filename: '[name].mjs'
    },

  },
  server: {
    port: lazyPort,
  }
});