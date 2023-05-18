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
    script: {
      plugins: ['swc-plugin-vue-jsx']
    }
  },
  plugins: [jsPluginVue()],
};
