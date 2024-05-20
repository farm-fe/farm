/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
  compilation: {
    input: {
      index: './src/index.ts'
    },
    output: {
      path: 'build',
      entryFilename: '[entryName].cjs',
      targetEnv: 'node',
      format: 'cjs'
    },
    external: ['typescript', 'fast-glob', 'ts-morph', 'fs-extra'],
    partialBundling: {
      enforceResources: [
        {
          name: 'index.js',
          test: ['.+']
        }
      ]
    },
    minify: false,
    sourcemap: false,
    persistentCache: false,
    presetEnv: false
  }
};
