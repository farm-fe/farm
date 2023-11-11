import { resolve } from "node:path";
import type { UserConfig } from "@farmfe/core";
import farmJsPluginLess from "@farmfe/js-plugin-less";
import farmJsPluginSvgr from "@farmfe/js-plugin-svgr";

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
        "react-dom": resolve(process.cwd(), "./node_modules/react-dom"),
        "react": resolve(process.cwd(), "./node_modules/react"),
        // mockjs: resolve(process.cwd(), "./patches/mock.js"),
      },
    },
    output: {
      path: "./build",
      filename: "assets/[resourceName].[contentHash].[ext]",
      assetsFilename: "static/[resourceName].[contentHash].[ext]",
    },
    partialBundling: {
      targetMinSize: 1024 * 2,
    },
  },
  server: {
    cors: true,
    port: 6260,
  },
  plugins: [
    "@farmfe/plugin-react",
    farmJsPluginLess(),
    farmJsPluginSvgr(),
  ],
});
