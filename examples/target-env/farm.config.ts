import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    output: {
      targetEnv: 'browser-esnext',
    },
    persistentCache: false,
    minify: false
  },
  plugins: ['@farmfe/plugin-react']
});
