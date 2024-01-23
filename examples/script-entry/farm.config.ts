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
      path: 'dist/esm',
      entryFilename: '[entryName].mjs',
      targetEnv: 'node',
      format: 'esm'
    },
    external: [
      ...builtinModules.map((m) => `^node:${m}$`),
      ...builtinModules.map((m) => `^${m}$`)
    ],
    minify: false,
    // partialBundling: {
    //   enforceResources: [
    //     {
    //       name: 'node.bundle.js',
    //       test: ['.+']
    //     }
    //   ]
    // }
  },
  server: {
    hmr: false
  }
};
