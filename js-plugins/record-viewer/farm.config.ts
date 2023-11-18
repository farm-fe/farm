import { builtinModules } from 'module';
// import type { UserConfig } from '@farmfe/core';
import farmDtsPlugin from '@farmfe/js-plugin-dts';

function defineConfig(config) {
  return config;
}

export default defineConfig({
  compilation: {
    input: {
      index: './src/index.ts'
    },
    output: {
      path: 'build/' + (process.env.FARM_FORMAT || 'cjs'),
      entryFilename: '[entryName].cjs',
      targetEnv: 'node',
      format: process.env.FARM_FORMAT || 'cjs'
    },
    external: [
      ...builtinModules.map((m) => `^${m}$`),
      ...builtinModules.map((m) => `^node:${m}$`)
    ],
    partialBundling: {
      enforceResources: [
        {
          name: 'index.js',
          test: ['.+']
        }
      ]
    },
    minify: false,
    sourcemap: false,
    presetEnv: false
  },
  server: {
    hmr: false
  },
  plugins: [
    farmDtsPlugin({
      tsConfigPath: './tsconfig.build.json'
    })
  ]
});
