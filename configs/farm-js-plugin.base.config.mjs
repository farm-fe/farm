import { builtinModules } from "module";

export function createFarmJsPluginBuildConfig(plugins, options = {}) {
  return {
    compilation: {
      input: {
        index: "./src/index.ts",
      },
      output: {
        path: "build",
        targetEnv: "library",
        format: ["esm", "cjs"],
      },
      external: [
        "farm",
        ...builtinModules.map((m) => `^${m}$`),
        ...builtinModules.map((m) => `^node:${m}$`),
        ...(options.external || []),
      ],
      progress: false,
      minify: false,
      sourcemap: false,
      comments: true,
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
