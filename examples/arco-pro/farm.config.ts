import { resolve } from 'node:path';
import { defineConfig } from '@farmfe/core';
import farmJsPluginLess from '@farmfe/js-plugin-less';
import svgr from '@farmfe/js-plugin-svgr';

export default defineConfig((env) => {
  return {
    compilation: {
      input: {
        index: './index.html'
      },
      sourcemap: true,
      presetEnv: false,
      concatenateModules: true,
      persistentCache: false,
      resolve: {
        alias: {
          '@': resolve(process.cwd(), './src'),
          'react-dom': resolve(process.cwd(), './node_modules/react-dom'),
          react: resolve(process.cwd(), './node_modules/react')
        }
      },
    },
    plugins: [
      '@farmfe/plugin-react',
      // '@farmfe/plugin-svgr',
      svgr(),
      farmJsPluginLess(),
    ]
  };
});
