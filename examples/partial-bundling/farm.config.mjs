import { builtinModules } from 'module';

/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
  compilation: {
    input: {
      pageA: './src/pageA.ts',
      pageB: './src/pageB.ts',
      pageC: './src/pageC.ts'
    },
    output: {
      path: 'dist',
      targetEnv: 'node',
      format: 'cjs'
    },
    external: [
      ...builtinModules.map((m) => `^node:${m}$`),
      ...builtinModules.map((m) => `^${m}$`)
    ],
    minify: false,
    presetEnv: false,
    sourcemap: false
  },
  server: {
    hmr: false
  },
  plugins: [
    // If you comment, you will use the default partial bundling
    [
      '@farmfe/plugin-webpack-partial-bundling',
      {
        // moduleBucket: [
        //   {
        //     name: 'common',
        //     test: [],
        //     // minSize: 1024
        //   }
        // ]
      }
    ]
  ]
};
