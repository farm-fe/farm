import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    sourcemap: false,
    persistentCache: false,
    presetEnv: false
  },
  // server: {
  //   open: true
  // },
  plugins: [
    ['@farmfe/plugin-react', { runtime: 'automatic' }],
    '@farmfe/plugin-sass',
  ],
});
