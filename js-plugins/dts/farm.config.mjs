/**
 * @type {import('farm').UserConfig}
 */
export default {
  compilation: {
    input: {
      index: "./src/index.ts",
    },
    output: {
      path: "build",
      entryFilename: "[entryName].cjs",
      targetEnv: "library",
      format: "cjs",
    },
    external: ["typescript", "fast-glob", "ts-morph", "fs-extra"],
    minify: false,
    sourcemap: false,
    persistentCache: false,
    presetEnv: false,
  },
};
