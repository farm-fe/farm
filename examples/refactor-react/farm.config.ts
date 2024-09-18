import { defineConfig } from "@farmfe/core";

import react from "@farmfe/plugin-react";
export default defineConfig({
  plugins: [
    react(),
    myCustomPlugin()
  ],
  compilation: {
    persistentCache: false,
    progress: false,
    output: {
      // publicPath: "/aaa/",
    },
  },
  server: {},
});


function myCustomPlugin() {
  return {
    name: 'vite-plugin-custom',
    updateModules: {
      executor(data) {
        console.log(data, "更新的模块");
      },
    },
  }
}
