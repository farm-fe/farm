import { defineConfig } from '@farmfe/core';
import vue from '@vitejs/plugin-vue';
import vueJsx from '@vitejs/plugin-vue-jsx';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig(({ mode }) => {
  return {
    server: {
      port: 3001,
      hmr: {
        port: 3002,
        host: 'localhost',
        protocol: 'ws'
      }
    },
    compilation: {
      input: {
        index: './index.html'
      },
      persistentCache: true,
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
        vitePlugin: vue(),
        filters: ['\\.vue', '\\\\0.+']
      }),
      vueJsx(),
      tailwindcss()
    ],
    plugins: []
  };
});
