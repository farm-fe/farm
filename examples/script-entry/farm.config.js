// change to @farmfe/core/config when resolve support conditional exports
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
    external: builtinModules,
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
