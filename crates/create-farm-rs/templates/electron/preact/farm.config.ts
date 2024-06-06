import { defineConfig } from '@farmfe/core';
import electron from '@farmfe/js-plugin-electron';
import preact from '@preact/preset-vite';

export default defineConfig({
  compilation: {
    persistentCache: false,
  },
  vitePlugins: [preact()],
  plugins: [
    electron({
      main: {
        input: 'electron/main.ts',
      },
      preload: {
        input: 'electron/preload.ts',
      },
    }),
  ]
});
