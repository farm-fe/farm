import { defineConfig } from "@farmfe/core";

import react from "@farmfe/plugin-react";
export default defineConfig({
  plugins: [
    react(),
    // myCustomPlugin(),
    compilerPlugin(),
  ],
  compilation: {
    // persistentCache: false,
    output: {
      // publicPath: "/aaa/",
    },
  },
  server: {},
});

function myCustomPlugin() {
  return {
    name: "custom",
    updateModules: {
      executor(data: any) {
        console.log(data, "更新的模块");
      },
    },
  };
}

function compilerPlugin() {
  return {
    name: "compiler",
    configureCompiler(compiler: any) {
      console.log(compiler, "compiler");
    },
  };
}
