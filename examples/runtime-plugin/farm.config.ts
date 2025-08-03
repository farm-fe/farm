import { defineConfig } from 'farm';

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
    lazyCompilation: false,
    minify: false,
    sourcemap: false,
    persistentCache: false,
    runtime: {
      plugins: ['./src/runtime-plugin.ts']
    }
  }
});
