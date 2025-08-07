import { defineConfig } from '@farmfe/core';
import Vue from '@vitejs/plugin-vue';

export default defineConfig(({ mode }) => {
  return {
    server: {
      port: 3001
    },
    compilation: {
      input: {
        index: './index.html'
      },
      persistentCache: false,
      output: {
        path: 'dist',
        publicPath: '/',
        targetEnv: 'browser-esnext',
        filename: '[ext]/[name]-[hash].[ext]',
        assetsFilename: 'assets/[name]-[hash].[ext]',
        format: 'cjs'
      },
      runtime: {
        isolate: true
      },
      resolve: {
        extensions: [
          'vue',
          'tsx',
          'ts',
          'jsx',
          'js',
          'mjs',
          'json',
          'html',
          'css'
        ]
      }
    },
    vitePlugins: [
      () => ({
        vitePlugin: Vue(),
        filters: ['.+\\.vue', '^\\\\0.+']
      })
    ],
    plugins: []
  };
});
