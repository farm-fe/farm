import { defineConfig } from '@farmfe/core';
import vue from '@vitejs/plugin-vue';
import vueDevTools from 'vite-plugin-vue-devtools';

export default defineConfig({
  vitePlugins: [vue(), vueDevTools()]
});
