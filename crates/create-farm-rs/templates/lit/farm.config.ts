import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    presetEnv: false,
    script: {
      plugins: [],
      target: 'es2022',
      parser: {
        tsConfig: {
          decorators: true,
          dts: false,
          noEarlyErrors: false,
          tsx: false
        }
      }
    }
  }
});
