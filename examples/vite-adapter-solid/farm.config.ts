import { defineConfig } from "farm";
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
  vitePlugins: [() => ({ vitePlugin: solid(), filters: [
    '\\.jsx$',
    '\\.tsx$',
  ] })],
});
