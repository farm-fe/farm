import { resolve } from 'node:path';
import { defineConfig } from '@farmfe/core';
import farmJsPluginLess from '@farmfe/js-plugin-less';
import farmJsPluginSvgr from '@farmfe/js-plugin-svgr';

export default defineConfig({
  compilation: {
    input: {
      index: './index.html'
    },
    sourcemap: true,
    resolve: {
      symlinks: true,
      alias: {
        '@': resolve(process.cwd(), './src'),
        'react-dom': resolve(process.cwd(), './node_modules/react-dom'),
        react: resolve(process.cwd(), './node_modules/react')
        // mockjs: resolve(process.cwd(), "./patches/mock.js"),
      }
    },
    output: {
      path: './build',
      filename: 'assets/[resourceName].[contentHash].[ext]',
      assetsFilename: 'static/[resourceName].[contentHash].[ext]'
    },
    partialBundling: {
      targetMinSize: 1024 * 2
    },
    persistentCache: false
  },
  server: {
    cors: true,
    port: 6260
  },
  plugins: [['@farmfe/plugin-react', {
    refresh: process.env.NODE_ENV === 'development',
    development: process.env.NODE_ENV === 'development'
  }], farmJsPluginLess(), farmJsPluginSvgr()]
});
