import type { UserConfig } from '@farmfe/core';
import farmJsPluginVue from '@farmfe/js-plugin-vue';

export default <UserConfig>{
  compilation: {
    input: {
      index: './index.html'
    },
    output: {
      path: './build'
    },
    resolve: {
      strictExports: true
    }
  },
  server: {
    proxy: {
      '/api': {
        target: 'http://v.juhe.cn/todayOnhistory/queryEvent.php',
        changeOrigin: true,
        rewrite: (path: any) => path.replace(/^\/api/, '')
      }
    }
  },
  plugins: [farmJsPluginVue()]
};
