// change to @farmfe/core/config when resolve support conditional exports
import { defineFarmConfig } from '@farmfe/core/config';
import farmJsPluginVue from '@farmfe/js-plugin-vue';
import farmPluginVueJsx from '@farmfe/plugin-vue-jsx';

export default defineFarmConfig({
  compilation: {
    input: {
      index: './index.html',
    },
    output: {
      path: './build',
    },
    resolve: {
      strictExports: true,
    },
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
  plugins: [farmJsPluginVue(), '@farmfe/plugin-vue-jsx'],
});
