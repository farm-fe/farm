import { defineConfig } from 'farm';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
  vitePlugins: [vue()],
  server: {
    port: 1420
  }
});
