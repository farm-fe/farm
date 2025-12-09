import { defineConfig, VERSION } from '@farmfe/core';
import dts from '@farmfe/js-plugin-dts';

export default defineConfig({
  compilation: {
    external: ['@farmfe/core'],
    input: {
      'qiankun-farm-plugin-helper': './src/qiankun-farm-plugin-helper.ts'
    },
    output: {
      targetEnv: 'library',
      path: `./dist/helper`,
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
