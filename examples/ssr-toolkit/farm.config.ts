import { defineConfig } from '@farmfe/core';
import farmLessPlugin from '@farmfe/js-plugin-less';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
  plugins: [farmLessPlugin()],
  vitePlugins: [vue()],
  server: {
    middlewareMode: true
  }
});
