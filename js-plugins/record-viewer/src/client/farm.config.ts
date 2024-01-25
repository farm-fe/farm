import farmPostcssPlugin from '@farmfe/js-plugin-postcss';
import vitejsPluginVue from '@vitejs/plugin-vue'
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  plugins: [farmPostcssPlugin()],
  vitePlugins: [vitejsPluginVue()],
  compilation: {
    record: true,
    output: {
      path: '../../build/client'
    },
  },
});
