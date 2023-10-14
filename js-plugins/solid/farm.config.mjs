import { builtinModules } from 'module';
import farmDtsPlugin from '@farmfe/js-plugin-dts';

/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
  compilation: {
    minify: false,
    presetEnv: false,
    input: {
      index: './src/index.ts'
    },
    output: {
      entryFilename: '[entryName].cjs',
      targetEnv: 'node',
      format: 'cjs'
    },
    external: [
      ...builtinModules.map((m) => `^${m}$`),
      ...builtinModules.map((m) => `^node:${m}$`)
    ],
    partialBundling: {
      enforceResources: [
        {
          name: 'index.js',
          test: ['.+']
        }
      ]
    },
    sourcemap: false,
    presetEnv: false
  },
  server: {
    hmr: false
  },
  // plugins: [farmDtsPlugin()]
};
