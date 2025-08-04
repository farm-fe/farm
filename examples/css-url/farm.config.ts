import { resolve } from 'node:path';
import { defineConfig } from 'farm';
import postcss from '@farmfe/js-plugin-postcss';

console.log(__dirname);
console.log(__filename);
console.log(import.meta.url);

export default defineConfig({
  compilation: {
    input: {
      index: './index.html'
    },
    persistentCache: false,
    output: {
      path: './build',
      publicPath: '/public-dir/'
    },
    assets: {
      include: ['aaa']
    },
    // record: true,
    sourcemap: true,
    // treeShaking: true,
    // minify: true,
    resolve: {
      alias: {
        '/@': resolve(__dirname, 'src'),
        '@': resolve(__dirname, 'src')
      }
    }
  },
  server: {},
  plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass', postcss()]
});
