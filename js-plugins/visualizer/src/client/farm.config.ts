import farmPostcssPlugin from '@farmfe/js-plugin-postcss';
import vitejsPluginVue from '@vitejs/plugin-vue'
import { defineConfig } from '@farmfe/core';
import record from '../dev';

export default defineConfig({
  plugins: [
    farmPostcssPlugin(), 
    ...(process.env.NODE_ENV === 'development' ? [record()] : [])
  ],
  vitePlugins: [vitejsPluginVue()],
  compilation: {
    output: {
      path: '../../build/client'
    },
  },
});
