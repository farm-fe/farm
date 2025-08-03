import { defineConfig } from 'farm';

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
