import { builtinModules } from 'module';

/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
  compilation: {
    output: {
      path: 'dist',
      filename: 'index.[ext]',
      format: 'cjs',
      targetEnv: 'node'
    },
    external: [
      ...builtinModules.map((m) => `^${m}$`),
      ...builtinModules.map((m) => `^node:${m}$`),
      '@farmfe/core'
    ],
    partialBundling: {
      enforceResources: [
        {
          name: 'node.bundle.js',
          test: ['.+']
        }
      ]
    },
    minify: false,
    sourcemap: false,
    presetEnv: false,
    treeShaking: false
  },
  server: {
    hmr: false
  }
};
