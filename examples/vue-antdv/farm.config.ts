import { defineConfig } from 'farm';
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
    },
    lazyCompilation: false,
  },
  plugins: [farmJsPluginLess()],
  vitePlugins: [Vue()]
});
