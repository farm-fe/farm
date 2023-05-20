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
      plugins: [{
        name: 'swc-plugin-vue-jsx',
        options: {
          "transformOn": true,
          "optimize": true
        },
        filters: {
          moduleTypes: ['tsx', 'jsx'],
        }
      }]
    }
  },
  plugins: [jsPluginVue()],
};
