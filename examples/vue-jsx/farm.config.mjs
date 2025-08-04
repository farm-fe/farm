import vue from "@vitejs/plugin-vue";
import jsx from "@vitejs/plugin-vue-jsx";

/**
 * @type {import('farm').UserConfig}
 */
export default {
  compilation: {
    input: {
      index: "index.html",
    },
    output: {
      path: "./build",
    },
    resolve: {
      strictExports: true,
    },
    presetEnv: false,
    // script: {
    //   plugins: [{
    //     name: 'swc-plugin-vue-jsx',
    //     options: {
    //       "transformOn": true,
    //       "optimize": true
    //     },
    //     filters: {
    //       moduleTypes: ['tsx', 'jsx'],
    //     }
    //   }]
    // }
  },
  vitePlugins: [vue(), jsx()],
};
