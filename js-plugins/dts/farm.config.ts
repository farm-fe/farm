import { builtinModules } from 'module';

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
      filename: 'index.[ext]',
      targetEnv: 'node'
    },
    external: [
      ...builtinModules.map((m) => `^${m}$`),
      ...builtinModules.map((m) => `^node:${m}$`)
    ],
    partialBundling: {
      moduleBuckets: [
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
