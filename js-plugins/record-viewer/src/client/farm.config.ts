import type { UserConfig } from '@farmfe/core';
import farmPostcssPlugin from '@farmfe/js-plugin-postcss';
import vitejsPluginVue from '@vitejs/plugin-vue'


function defineConfig(config: UserConfig) {
  return config;
}

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
