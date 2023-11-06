// change to @farmfe/core/config when resolve support conditional exports
import { UserConfig } from '@farmfe/core';
import farmJsPluginVue from '@farmfe/js-plugin-vue';
import farmJsPluginLess from '@farmfe/js-plugin-less';
import vue from '@vitejs/plugin-vue';

import path from 'path';

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  compilation: {
    input: {
      index: './index.html'
    },
    output: {
      path: './build'
    },
    resolve: {
      strictExports: true,
      alias: {
        '/@': path.join(process.cwd(), 'src')
      }
    }
  },
  server: {
    proxy: {
      '/api': {
        target: 'https://music-erkelost.vercel.app/banner',
        changeOrigin: true,
        rewrite: (path: any) => path.replace(/^\/api/, '')
      }
    },
    cors: true
  },
  plugins: [farmJsPluginLess()],
  vitePlugins: [vue()]
});
