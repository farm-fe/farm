import { defineConfig } from '@farmfe/core';

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
    }
  }
});