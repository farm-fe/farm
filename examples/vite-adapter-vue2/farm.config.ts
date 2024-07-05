import { defineConfig } from '@farmfe/core';
import path from 'node:path';
import { createVuePlugin } from "vite-plugin-vue2";
import { createSvgPlugin } from 'vite-plugin-vue2-svg';

export default defineConfig({
  compilation: {
    persistentCache: false,
    resolve: {
      alias: {
        '@': path.resolve('src')
      }
    },
  },
  server: {
    proxy: {
      "/aaa": {
        target: "https://apis.juhe.cn/environment/river",
        changeOrigin: true,
        pathRewrite: { "^/aaa": "" }
      },
    },
  },
  vitePlugins: [createVuePlugin(), createSvgPlugin(), {
    name: 'custom-plugin',
    transform(code, id) {
      if (id.endsWith('.png')) {
        return {
          code,
          map: null
        }
      }
      return null;
    }
  }]
});
