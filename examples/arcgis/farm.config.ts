import { defineConfig } from "@farmfe/core";
import visualizer from "@farmfe/js-plugin-visualizer";

export default defineConfig({
  compilation: {
    persistentCache: false,
    // lazyCompilation: false,
    // partialBundling: {
    //   enforceResources: [
    //     {
    //       name: "index",
    //       test: [".*"]
    //     }
    //   ]
    // }
  },
  server: {
    port: 9001
  },
  plugins: [
    visualizer()
  ]
})