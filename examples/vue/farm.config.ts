// change to @farmfe/core/config when resolve support conditional exports
import { defineFarmConfig } from '@farmfe/core/config';
import farmJsPluginVue from '@farmfe/js-plugin-vue';

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
  plugins: [farmJsPluginVue()],
});
