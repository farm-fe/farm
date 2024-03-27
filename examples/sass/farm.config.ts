import { defineConfig } from '@farmfe/core';
import farmSassPlugin from '@farmfe/js-plugin-sass';
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
  plugins: ['@farmfe/plugin-react',farmSassPlugin({
    additionalData: (content:string, resolvePath:string) => {
      if (path.basename(resolvePath, '.scss') === 'index') {
        return `$hoverColor: #f10215;`;
      }
    },
  }) ],
});
