import { defineConfig } from '@farmfe/core';
import farmJsPluginVue from '@farmfe/js-plugin-vue';
import sass from '@farmfe/js-plugin-sass';

export default defineConfig({
  compilation: {
    input: {
      index: './index.html'
    },
    output: {
      path: './build'
    }
  },
  plugins: [
    farmJsPluginVue(),
    sass({ legacy: true })
  ]
});
