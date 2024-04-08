import { defineConfig } from '@farmfe/core';
import Pages from 'vite-plugin-pages';

export default defineConfig({
  plugins: ['@farmfe/plugin-react'],
  vitePlugins:[
   Pages({
      resolver:'react'
    }),
  ]
});
