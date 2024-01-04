// vite.config.ts
import UnoCSS from 'unocss/vite'
import vue from '@vitejs/plugin-vue';
import { defineConfig } from 'vite'

export default defineConfig({
  plugins: [
    vue(),
    UnoCSS(),
  ],
})