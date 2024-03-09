import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    input: {
      index: './src/index.ts'
    },
    output: {
      path: 'dist',
      entryFilename: '[entryName].mjs',
      targetEnv: 'node',
      format: 'esm'
    },
    mode: 'development',
    minify: false,
    sourcemap: false,
    persistentCache: false,
    runtime: {
      plugins: ['./src/runtime-plugin.ts']
    }
  },
  server: {
    hmr: false
  }
});
