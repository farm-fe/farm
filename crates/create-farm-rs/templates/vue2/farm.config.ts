import { defineConfig } from 'farm';
import vue2 from '@vitejs/plugin-vue2'

export default defineConfig({
  vitePlugins: [
    vue2(),
  ]
});
