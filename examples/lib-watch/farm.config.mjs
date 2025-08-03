/**
 * @type {import('farm').UserConfig}
 */
export default {
  compilation: {
    input: {
      index: "./index.ts",
    },
    output: {
      path: "dist",
      filename: "[resourceName].[ext]",
      targetEnv: "node",
    },
    lazyCompilation: false,
    // partialBundling: {
    //   enforceResources: [
    //     {
    //       name: 'node.bundle.js',
    //       test: ['.+']
    //     }
    //   ]
    // },
    minify: false,
    sourcemap: false,
    presetEnv: false,
  },
  server: {
    hmr: false,
  },
};
