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
      path: 'build',
      filename: 'index.[ext]'
    },
    external: builtinModules.map((m) => `^${m}$`),
    partialBundling: {
      moduleBuckets: [
        {
          name: 'node.bundle.js',
          test: ['.+']
        }
      ]
    }
  },
  server: {
    hmr: false
  }
};
