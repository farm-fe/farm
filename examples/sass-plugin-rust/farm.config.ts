import { defineConfig } from '@farmfe/core';
import path from 'path';

export default defineConfig({
  compilation: {
    input: {
      index: './index.html',
    },
    resolve: {
      symlinks: true,
      alias: {
        '@': path.resolve('src')
      }
    },
    define: {
      BTN: 'Click me',
    },
    output: {
      path: './build',
    },
    sourcemap: false,
    persistentCache: false,
    // treeShaking: true,
    // minify: true,
  },
  server: {
    hmr: true,
  },
  plugins: [
    '@farmfe/plugin-react',
    ['@farmfe/plugin-sass', {
      additionalData:  `$hoverColor: #f10215;`
    }] 
  ],
});
