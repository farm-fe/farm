import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    persistentCache: false,
    presetEnv: false,
  },
  server: {
    port: 4878
  },
  plugins: [
    ['@farmfe/plugin-react', { runtime: 'automatic' }],
    '@farmfe/plugin-sass',
  ],
});
