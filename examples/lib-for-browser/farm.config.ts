import { defineConfig } from '@farmfe/core';
import farmJsPluginDts from '@farmfe/js-plugin-dts';

export default defineConfig({
  compilation: {
    input: {
      index: 'src/index.ts'
    },
    output: {
      targetEnv: 'browser',
      format: 'esm',
      path: 'dist',
      entryFilename: '[entryName].js',
      filename: '[name].jsx'
    },
    minify: false,
    presetEnv: false,
    partialBundling: {
      enforceResources: [
        {
          name: 'components',
          test: ['src/components/.+']
        }
      ]
    }
  },
  plugins: [farmJsPluginDts({})]
});
