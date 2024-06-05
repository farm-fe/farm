import { defineConfig } from '@farmfe/core';
import Pages from 'vite-plugin-pages';

export default defineConfig({
  plugins: ['@farmfe/plugin-react'],
  compilation:{
    persistentCache:false,
  },
  vitePlugins:[
   Pages({
      resolver:'react'
    }),
  ]
});
