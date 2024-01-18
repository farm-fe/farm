import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    output: {
      targetEnv: 'node'
    },
    persistentCache: false,
    sourcemap: false,
    presetEnv: false
  }
});
