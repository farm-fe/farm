import { resolve } from "node:path";
import { defineConfig } from "@farmfe/core";
import farmJsPluginLess from "@farmfe/js-plugin-less";

export default defineConfig((env) => {
  return {
    compilation: {
      resolve: {
        alias: {
          "@": resolve(process.cwd(), "./src"),
        },
        dedupe: ["react", "react-dom"],
      },
    },
    plugins: [
      "@farmfe/plugin-react",
      "@farmfe/plugin-svgr",
      farmJsPluginLess(),
    ],
  };
});
