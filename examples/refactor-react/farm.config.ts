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
    }
  },
  server: {},
});


function myCustomPlugin() {
  return {
    name: 'custom',
    updateModules: {
      executor(data: any) {
        console.log(data, "更新的模块");
      },
    },
  }
}
