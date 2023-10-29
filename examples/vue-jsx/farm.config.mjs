import jsPluginVue from '@farmfe/js-plugin-vue';

/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
  compilation: {
    input: {
      index: 'index.html',
    },
    output: {
      path: './build',
    },
    resolve: {
      strictExports: true,
    },
    presetEnv: false,
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
