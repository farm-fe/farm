import { defineConfig } from '@farmfe/core';
import electron from '@farmfe/js-plugin-electron';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  compilation: {
    persistentCache: false,
  },
  vitePlugins: [
    svelte(),
  ],
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
