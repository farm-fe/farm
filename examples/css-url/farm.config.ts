import { resolve } from 'node:path';
import type { UserConfig } from '@farmfe/core';
import postcss from '@farmfe/js-plugin-postcss';

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  compilation: {
    input: {
      index: './index.html',
    },
    persistentCache: false,
    output: {
      path: './build',
      publicPath: '/public-dir/',
    },
    assets: {
      include: ['aaa']
    },
    record: true,
    sourcemap: true,
    // treeShaking: true,
    // minify: true,
    resolve:{
      alias: {
        '/@': resolve(__dirname, 'src'),
        '@': resolve(__dirname, 'src')
      },
    }
  },
  server: {
    open: true,
  },
  plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass', postcss()],
});
