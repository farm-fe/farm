import dts from '@farmfe/js-plugin-dts';
import { defineConfig } from 'farm';

export default defineConfig({
  compilation: {
    input: {
      index: './src/index.ts'
    },
    output: {
      targetEnv: 'library',
      format: ['esm', 'cjs']
    },
    external: [
      'farm',
      '@tailwindcss/node',
      '@tailwindcss/oxide',
      'lightningcss',
      'postcss',
      'postcss-import'
    ],
    resolve: {
      autoExternalFailedResolve: true,
      dedupe: ['tailwindcss']
    },
    mode: 'development',
    minify: false,
    lazyCompilation: false,
    treeShaking: false,
    persistentCache: false,
    progress: false
  },
  // plugins: ['@farmfe/plugin-dts']
  plugins: [dts()]
});
