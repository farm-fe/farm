import { builtinModules } from 'module';

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
      moduleBuckets: [
        {
          name: 'index.js',
          test: ['.+']
        }
      ]
    }
  },
  server: {
    hmr: false
  },
};
