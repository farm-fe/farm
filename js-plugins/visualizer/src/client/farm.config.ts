import { defineConfig } from '@farmfe/core';
import Pages from 'vite-plugin-pages';
import { visualizer } from '../server';

export default defineConfig({
  plugins: ['@farmfe/plugin-react', visualizer()],
  vitePlugins: [
    Pages({
      resolver: 'react',
      dirs: 'pages'
    })
  ]
});
