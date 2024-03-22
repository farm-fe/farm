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
  plugins: [farmJsPluginVue()]
});
