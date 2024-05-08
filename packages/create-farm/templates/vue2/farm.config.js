import { defineConfig } from '@farmfe/core';
import vue2 from '@vitejs/plugin-vue2'

export default defineConfig({
  vitePlugins: [
    vue2(),
  ]
});
