// change to @farmfe/core/config when resolve support conditional exports
import { UserConfig } from '@farmfe/core';
import farmJsPluginVue from '@farmfe/js-plugin-vue';
import path from 'path';

export default <UserConfig>{
  compilation: {
    input: {
      index: './index.html',
    },
    output: {
      path: './build',
    },
    resolve: {
      strictExports: true,
      alias: {
        '/@': path.join(process.cwd(), 'src')
      }
    },
    lazyCompilation: true
  },
  server: {
    proxy: {
      '/api': {
        target: 'https://music-erkelost.vercel.app/banner',
        changeOrigin: true,
        rewrite: (path: any) => path.replace(/^\/api/, ''),
      },
    },
  },
  plugins: [farmJsPluginVue()],
};
