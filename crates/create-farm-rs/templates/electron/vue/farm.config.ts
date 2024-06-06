import { defineConfig } from '@farmfe/core';
import electron from '@farmfe/js-plugin-electron';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
  compilation: {
    persistentCache: false,
  },
  vitePlugins: [
    vue(),
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
