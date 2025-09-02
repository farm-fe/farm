import { defineConfig } from 'farm';
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
