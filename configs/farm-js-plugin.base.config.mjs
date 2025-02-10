import { builtinModules } from "module";

const format = process.env.FARM_FORMAT || "cjs";
const ext = format === "esm" ? "mjs" : "cjs";

export function createFarmJsPluginBuildConfig(plugins, options = {}) {
  return {
    compilation: {
      input: {
        index: "./src/index.ts",
      },
      output: {
        path: `build/${format}`,
        entryFilename: `[entryName].${ext}`,
        targetEnv: "node",
        format,
      },
      external: [
        "@farmfe/core",
        ...builtinModules.map((m) => `^${m}$`),
        ...builtinModules.map((m) => `^node:${m}$`),
        ...(options.external || []),
      ],
      partialBundling: {
        enforceResources: [
          {
            name: "index.js",
            test: [".+"],
          },
        ],
      },
      progress: false,
      minify: false,
      sourcemap: false,
      presetEnv: false,
      persistentCache: false,
      lazyCompilation: false,
      // persistentCache: {
      //   envs: {
      //     FARM_FORMAT: format
      //   }
      // }
    },
    server: {
      hmr: false,
    },
    plugins,
  };
}
