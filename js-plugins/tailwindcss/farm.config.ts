import { defineConfig } from '@farmfe/core';
import dts from '@farmfe/js-plugin-dts';

export default defineConfig({
  compilation: {
    input: {
      index: './src/index.ts'
    },
    output: {
      targetEnv: 'node',
      format: 'esm'
    },
    external: ['@farmfe/core', '@tailwindcss/oxide', 'lightningcss'],
    resolve: {
      autoExternalFailedResolve: true,
      dedupe: ['tailwindcss']
    },
    mode: 'development',
    minify: false,
    lazyCompilation: false,
    treeShaking: false
  },
  plugins: [dts()]
});
