import { defineConfig } from '@farmfe/core';
import path from 'path';
import { cwd } from 'process';
import Pages from 'vite-plugin-pages';

export default defineConfig({
  root: cwd(),
  plugins: ['@farmfe/plugin-react'],
  compilation:{
    resolve:{
      alias: {
        "/src": path.resolve(process.cwd(), "src"),
      }
    }
  },
  vitePlugins:[
   Pages({
      resolver:'react',
      moduleId:"~react-page",
      extensions: ['tsx', 'jsx', 'ts', 'js'],
    }),
  ]
});
