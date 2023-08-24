import { builtinModules } from 'module';

/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
  compilation: {
    input: {
      pageA: './src/pageA.ts',
      pageB: './src/pageB.ts',
      pageC: './src/pageC.ts'
    },
    output: {
      path: 'dist',
      targetEnv: 'node',
      format: 'cjs'
      // entryFilename: '[entryName].mjs'
    },
    external: [
      ...builtinModules.map((m) => `^node:${m}$`),
      ...builtinModules.map((m) => `^${m}$`)
    ],
    minify: false,
    presetEnv: false,
    sourcemap: false,
    lazyCompilation: false,
    partialBundling: {
      moduleBuckets: [
        {
          name: 'common',
          test: [],
          minSize: 1024,
          maxConcurrentRequests: 5
        }
      ]
    }
  },
  server: {
    hmr: false
  }
};
