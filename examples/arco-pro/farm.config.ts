import { resolve } from "node:path";
import type { UserConfig } from "@farmfe/core";
import farmJsPluginLess from "@farmfe/js-plugin-less";
function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  compilation: {
    input: {
      index: "./index.html",
    },
    lazyCompilation: true,
    presetEnv: false,
    resolve: {
      symlinks: true,
      alias: {
        "@": resolve(process.cwd(), "./src"),
        // mockjs: resolve(process.cwd(), "./patches/mock.js"),
      },
    },
    output: {
      path: "./build",
      filename: "assets/[resourceName].[contentHash].[ext]",
      assetsFilename: "static/[resourceName].[contentHash].[ext]",
    },
  },
  server: {
    cors: true,
    port: 6260,
    host: "localhost",
  },
  plugins: [
    "@farmfe/plugin-react",
    farmJsPluginLess(),
  ],
});
