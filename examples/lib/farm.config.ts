import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    presetEnv: false,
    progress: false,
    input: {
      index: "./index.ts"
    },
    persistentCache: false
  },
  plugins: ['@farmfe/plugin-dts']
});
