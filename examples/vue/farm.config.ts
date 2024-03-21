import { defineConfig } from '@farmfe/core';
import farmJsPluginVue from '@farmfe/js-plugin-vue';

export default defineConfig({
  compilation: {
    input: {
      index: './index.html'
    },
    output: {
      path: './build'
    }
  },
  server: {
    proxy: {
      '^/api': {
        target: 'https://music-erkelost.vercel.app/banner',
        changeOrigin: true,
        rewrite: (path: any) => path.replace(/^\/api/, '')
      }
    }
  },
  plugins: [farmJsPluginVue()]
});
