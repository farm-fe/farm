import { defineConfig } from '@farmfe/core';
import farmDtsPlugin from '@farmfe/js-plugin-dts';

const format = (process.env.FARM_FORMAT as 'esm' | 'cjs') || 'esm';
const ext = format === 'esm' ? 'mjs' : 'cjs';

export default defineConfig({
  compilation: {
    input: {
      index: './src/index.ts'
    },
    output: {
      path: `build/${format}`,
      entryFilename: `[entryName].${ext}`,
      targetEnv: 'node',
      format
    },
    partialBundling: {
      enforceResources: [
        {
          name: 'index',
          test: ['.+']
        }
      ]
    },
    minify: false,
    sourcemap: false,
    presetEnv: false,
    persistentCache: {
      envs: {
        FARM_FORMAT: format
      }
    }
  },
  server: {
    hmr: false
  },
  plugins: [
    farmDtsPlugin()
  ]
});