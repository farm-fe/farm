// change to @farmfe/core/config when resolve support conditional exports
import { defineFarmConfig } from '@farmfe/core/dist/config';
import farmJsPluginVue from '@farmfe/js-plugin-vue';

export default defineFarmConfig({
  compilation: {
    input: {
      index: './index.html'
    },
    output: {
      path: './build'
    }
  },
  plugins: [farmJsPluginVue()]
});
