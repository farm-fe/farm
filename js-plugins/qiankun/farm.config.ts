import { defineConfig } from '@farmfe/core';
import dts from '@farmfe/js-plugin-dts';

export default defineConfig({
  compilation: {
    external: ['@farmfe/core'],
    input: {
      'qiankun-farm-plugin': './src/qiankun-farm-plugin.ts'
    },
    output: {
      targetEnv: 'library',
      path: `./dist/plugin`,
      format: ['esm', 'cjs']
    },
    minify: false,
    sourcemap: false,
    resolve: {
      autoExternalFailedResolve: true
    }
  },
  plugins: [dts()]
});
