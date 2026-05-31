import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    input: {
      index: './index.html',
    },
    output: {
      path: './build',
    },
    persistentCache: {
      cacheDir: 'node_modules/.farm/vue-cache',
    },
  },
  server: {
    hmr: true,
  },
  plugins: [
    '@farmfe/plugin-vue',
    '@farmfe/plugin-vue-jsx',
    '@farmfe/plugin-sass',
  ],
});
