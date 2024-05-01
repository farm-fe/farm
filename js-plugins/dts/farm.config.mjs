import { builtinModules } from "module";

/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
  compilation: {
    input: {
      index: "./src/index.ts"
    },
    output: {
      path: "build",
      entryFilename: "[entryName].cjs",
      targetEnv: "node",
      format: "cjs"
    },
    partialBundling: {
      enforceResources: [
        {
          name: "index.js",
          test: [".+"]
        }
      ]
    },
    minify: false,
    sourcemap: false,
    persistentCache: false,
    presetEnv: false
  }
};
