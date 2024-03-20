import { defineConfig } from '@farmfe/core';
import path from 'path';
import { cwd } from 'process';
import Pages from 'vite-plugin-pages';

export default defineConfig({
  plugins: ['@farmfe/plugin-react'],
  vitePlugins:[
   Pages({
      resolver:'react'
    }),
  ]
});
