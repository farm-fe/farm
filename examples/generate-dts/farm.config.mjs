import farmDtsPlugin from '@farmfe/js-plugin-dts';
import { builtinModules } from 'module';

/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
  compilation: {
    input: {
      index: 'src/index.ts'
    },
    output: {
      path: 'dist',
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
  },
  plugins: [farmDtsPlugin({
    outputDir: 'build'
  })]
};
