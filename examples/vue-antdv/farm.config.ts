import { defineConfig } from '@farmfe/core';
import Vue from '@vitejs/plugin-vue'
import farmJsPluginLess from '@farmfe/js-plugin-less';
import path from 'path';


export default defineConfig({
  compilation: {
    resolve: {
      strictExports: true,
      alias: {
        '/@': path.join(process.cwd(), 'src')
      }
    }
  },
  plugins: [farmJsPluginLess()],
  vitePlugins: [Vue()]
});
