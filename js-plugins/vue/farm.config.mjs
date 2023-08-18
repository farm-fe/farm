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
      entryFilename: '[entryName].cjs',
      targetEnv: 'node',
      format: 'cjs'
    },
    external: [
      ...builtinModules.map((m) => `^${m}$`),
      ...builtinModules.map((m) => `^node:${m}$`)
    ],
    partialBundling: {
      moduleBuckets: [
        {
          name: 'index.js',
          test: ['.+']
        }
      ]
    },
    minify: false,
    sourcemap: true,
    presetEnv: false
  },
  server: {
    hmr: false
  },
  plugins: [
    farmDtsPlugin({
      // libFolderPath: '../../node_modules/typescript/lib',
      // outputDir: ['dist', 'types'],
      // // include: ['src/index.ts'],
      // // aliasesExclude: [/^@components/],
      // staticImport: true,
      // skipDiagnostics: false,
      // // rollupTypes: true,
      // insertTypesEntry: true
    })
  ]
};
