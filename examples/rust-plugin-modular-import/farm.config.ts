import { defineConfig } from "@farmfe/core";
import path from 'node:path';
import vue from '@vitejs/plugin-vue2';
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
  vitePlugins: [vue()],
  plugins: [
    ['@farmfe/plugin-modular-import', {
      libraryName: 'element-ui',
      libDir: 'lib',
      camel2Dash: false,
      styleLibDir: 'lib',
      styleLibraryName: 'theme-chalk',
      styleLibraryPath: '.css'
    }]
  ],
});
