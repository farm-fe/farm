import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    presetEnv: false,
    123
  },
  server: {
    // port: 3000,
  },
  plugins: [
    ['@farmfe/plugin-react', { runtime: 'automatic' }],
    '@farmfe/plugin-sass',
  ],
});
