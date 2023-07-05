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
      filename: '[resourceName].[contentHash].cjs',
      targetEnv: 'node',
      format: 'cjs'
    },
    partialBundling: {
      moduleBuckets: [
        {
          name: 'index.js',
          test: ['.+']
        }
      ]
    },
    minify: false,
    sourcemap: false,
    presetEnv: false
  },
  server: {
    hmr: false
  }
};
