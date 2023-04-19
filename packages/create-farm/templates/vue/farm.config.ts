import type { UserConfig } from '@farmfe/core';
import farmJsPluginVue from '@farmfe/js-plugin-vue';

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
  plugins: [farmJsPluginVue()],
};
