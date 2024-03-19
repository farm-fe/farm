import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    sourcemap: false,
    persistentCache: false,
  },
  plugins: [
    ['@farmfe/plugin-react', { runtime: 'automatic' }],
    '@farmfe/plugin-sass',
  ],
});
