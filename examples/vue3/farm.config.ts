import { defineConfig } from '@farmfe/core';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
  vitePlugins: [
    vue(),
  ],
  compilation: {
    mode: 'production'
  },
  server: {
    port: 5232
  }
});
