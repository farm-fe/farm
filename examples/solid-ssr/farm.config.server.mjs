import { builtinModules } from "node:module";
import solid from "vite-plugin-solid";

/**
 * @type {import('farm').UserConfig}
 */
export default {
  compilation: {
    input: {
      index: "./src/server.tsx",
    },
    output: {
      path: "./dist",
      targetEnv: "node",
      format: "esm",
    },
    resolve: {},
    external: [...builtinModules.map((m) => `^${m}$`)],
    css: {
      prefixer: {
        targets: ["last 2 versions", "Firefox ESR", "> 1%", "ie >= 11"],
      },
    },
    treeShaking: false,
    persistentCache: false,
    minify: false,
    lazyCompilation: false,
  },
  vitePlugins: [
    () => ({
      filters: [".+"],
      vitePlugin: solid({ ssr: true }),
    }),
  ],
};
