import { builtinModules } from "module";

/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
  compilation: {
    input: {
      index: "./src/index.ts",
    },
    output: {
      path: "build",
      entryFilename: "[entryName].cjs",
      targetEnv: "node",
      format: "cjs",
    },
    external: [
      ...builtinModules.map((m) => `^${m}$`),
      ...builtinModules.map((m) => `^node:${m}$`),
      "@farmfe/core",
    ],
    partialBundling: {
      enforceResources: [
        {
          name: "index.js",
          test: [".+"],
        },
      ],
    },
    minify: false,
    sourcemap: false,
    presetEnv: false,
  },
  server: {
    hmr: false,
  },
};
