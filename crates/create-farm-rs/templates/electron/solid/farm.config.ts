import { defineConfig } from '@farmfe/core';
import electron from '@farmfe/js-plugin-electron';
import solid from 'vite-plugin-solid';

export default defineConfig({
  compilation: {
    persistentCache: false,
  },
  vitePlugins: [
    () => ({
      vitePlugin: solid(),
      filters: ['\\.tsx$', '\\.jsx$']
    })
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
