import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    persistentCache: false,
    output: {
      targetEnv: 'node'
    },
    resolve: {
      autoExternalFailedResolve: true
    },
    sourcemap: false,
    presetEnv: false,
    externalNodeBuiltins: false,
    external: ['@farmfe/core']
  }
});
