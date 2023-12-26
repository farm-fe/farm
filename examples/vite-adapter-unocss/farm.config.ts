import { defineConfig } from '@farmfe/core';
import vue from '@vitejs/plugin-vue';
import UnoCSS from 'unocss/vite'

export default defineConfig({
  compilation: {
    presetEnv: false
  },
  vitePlugins: [
    vue(),
    UnoCSS()
  ]
});
