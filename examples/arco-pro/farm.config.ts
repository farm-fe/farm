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
      output: {
        // showFileSize: false,
      },
      // persistentCache: false,
      resolve: {
        alias: {
          "@": resolve(process.cwd(), "./src"),
        },
        dedupe: ["react", "react-dom"],
      },
    },
    plugins: [
      '@farmfe/plugin-react',
      // '@farmfe/plugin-svgr',
      svgr(),
      farmJsPluginLess(),
    ],
  };
});
