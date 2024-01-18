import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    persistentCache: false,
    output: {
      targetEnv: 'node'
    },
    persistentCache: false,
    sourcemap: false,
    presetEnv: false
  }
});
