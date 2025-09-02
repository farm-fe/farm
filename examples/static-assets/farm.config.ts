import { defineConfig } from 'farm';
import sass from '@farmfe/js-plugin-sass';
import path from 'node:path';

export default defineConfig({
  compilation: {
    persistentCache: false,
    resolve: {
      symlinks: true,
      alias: {
        '@': path.resolve('src')
      }
    },
  },
  plugins: [
    sass({
      legacy: true
    })
  ]
})