import type { UserConfig } from '@farmfe/core';
import jsPluginVue from '@farmfe/js-plugin-vue';

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
    },
  },
  plugins: [jsPluginVue(), '@farmfe/plugin-vue-jsx'],
};
