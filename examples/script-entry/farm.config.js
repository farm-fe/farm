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
      filename: 'index.[ext]'
    },
    external: [
      ...builtinModules.map((m) => `^node:${m}$`),
      ...builtinModules.map((m) => `^${m}$`)
    ]
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
