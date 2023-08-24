import { builtinModules } from 'module';
import farmDtsPlugin from '@farmfe/js-plugin-dts';
/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
  compilation: {
    input: {
      index: './src/farm-vue-plugin.ts'
    },
    output: {
      path: 'build',
      entryFilename: '[entryName].[ext]',
      filename: '[resourceName].[ext]',
      targetEnv: 'node',
      format: 'cjs'
    },
    external: [
      ...builtinModules.map((m) => `^${m}$`),
      ...builtinModules.map((m) => `^node:${m}$`)
    ],
    partialBundling: {
      moduleBuckets: [
        // {
        //   name: 'index.js',
        //   test: ['.+']
        // }
      ]
    },
    minify: false,
    sourcemap: false,
    presetEnv: false
  },
  server: {
    hmr: false
  },
  plugins: [farmDtsPlugin({
    tsConfigPath: './tsconfig.json',
  })]
};
