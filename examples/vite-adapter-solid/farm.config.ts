import { defineConfig } from "@farmfe/core";
import solid from "vite-plugin-solid";

export default defineConfig({
  compilation: {
    input: {
      index: "index.html",
    },
    output: {
      path: "build",
    },
    define: {
      __DEV__: true,
    },
    persistentCache: false,
  },
  server: {
    open: true,
  },
  vitePlugins: [() => ({ vitePlugin: solid(), filters: [
    '\\.jsx$',
    '\\.tsx$',
  ] })],
});
