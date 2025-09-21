import { defineConfig } from '@farmfe/core';
import farmPostcssPlugin from '@farmfe/js-plugin-postcss';
import worker from '@farmfe/plugin-worker';
import vitejsPluginVue from '@vitejs/plugin-vue';
import record from '../dev';

export default defineConfig((env) => ({
  plugins: [
    farmPostcssPlugin(),
    ...(env.mode === 'development' ? [record()] : []),
    worker()
  ],
  vitePlugins: [vitejsPluginVue()],
  compilation: {
    concatenateModules: false,
    persistentCache: false,
    output: {
      path: '../../build/client'
    },
    external: ['@farmfe/core']
  }
}));
