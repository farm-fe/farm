import { defineConfig } from '@farmfe/core';
export default defineConfig({
  compilation: {
    output: {
      targetEnv: 'node'
    },
    external: ['@farmfe/core']
  }
});
