import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    persistentCache: false,
    output: {
      targetEnv: 'node'
    },
    sourcemap: false,
    presetEnv: false
  }
});
