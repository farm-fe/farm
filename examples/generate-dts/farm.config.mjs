import farmDtsPlugin from '@farmfe/js-plugin-dts';
import { builtinModules } from 'module';
import path from 'path';

/**
 * @type {import('farm').UserConfig}
 */
export default {
  compilation: {
    input: {
      index: 'src/index.ts'
    },
    resolve: {
      alias: {
        '@': path.resolve(process.cwd(), './src')
      }
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
    treeShaking: true
  },
  server: {
    hmr: false
  },
  plugins: [
    farmDtsPlugin({
      outputDir: 'build'
    })
  ]
};
