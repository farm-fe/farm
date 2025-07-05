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
        path: "build",
        entryFilename: `[entryName].${ext}`,
        targetEnv: "library",
        format: ["esm", "cjs"],
      },
      external: [
        "@farmfe/core",
        ...builtinModules.map((m) => `^${m}$`),
        ...builtinModules.map((m) => `^node:${m}$`),
        ...(options.external || []),
      ],
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
