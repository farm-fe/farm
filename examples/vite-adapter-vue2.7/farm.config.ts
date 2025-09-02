import { defineConfig } from 'farm';
import path from 'node:path';
import vue from '@vitejs/plugin-vue2'
import { createSvgPlugin } from 'vite-plugin-vue2-svg';

export default defineConfig({
  compilation: {
    // persistentCache: false,
    progress: false,
    resolve: {
      alias: {
        '@': path.resolve(process.cwd(), 'src'),
      }
    }
  },
  vitePlugins: [vue(), createSvgPlugin()]
});
