import { builtinModules } from 'module';

/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
  compilation: {
    input: {
      index: './index.ts'
    },
    output: {
      path: 'dist',
      targetEnv: 'node',
      entryFilename: '[entryName].mjs'
    },
    external: [
      ...builtinModules.map((m) => `^node:${m}$`),
      ...builtinModules.map((m) => `^${m}$`)
    ],
    minify: false,
    presetEnv: false,
    // partialBundling: {
    //   moduleBuckets: [
    //     {
    //       name: 'node.bundle.js',
    //       test: ['.+']
    //     }
    //   ]
    // }
  },
  server: {
    hmr: false
  }
};
