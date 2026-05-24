import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    input: {
      index: './index.html',
    },
    output: {
      path: './build',
    },
    persistentCache: false,
  },
  server: {
    hmr: true,
  },
  plugins: [
    '@farmfe/plugin-vue',
    '@farmfe/plugin-sass',
  ],
});
