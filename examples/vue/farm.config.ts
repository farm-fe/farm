import { defineConfig } from '@farmfe/core';
import vue from '@farmfe/plugin-vue';

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
    vue({
      script: {
        babelParserPlugins: ['deferredImportEvaluation']
      }
    }),
    '@farmfe/plugin-sass',
  ],
});
