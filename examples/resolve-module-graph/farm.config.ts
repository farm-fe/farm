import { builtinModules } from 'module';

export default {
  compilation: {
    input: {
      index: './index.ts'
    },
    output: {
      path: 'dist',
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
    treeShaking: true
  },
  server: {
    hmr: false
  }
};
