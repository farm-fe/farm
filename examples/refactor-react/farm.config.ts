import { defineConfig } from "@farmfe/core";

import react from "@farmfe/plugin-react";
import path from "path";

console.log(__dirname);

export default defineConfig({
  plugins: [
    react(),
    // myCustomPlugin(),
    compilerPlugin(),
  ],
  compilation: {
    // persistentCache: false,
    persistentCache: {
      cacheDir: "node_modules/.adny",
    },
    output: {
      // publicPath: "/aaa/",
    },
    resolve: {
      // alias: {
      //   "@": path.resolve("src"),
      // },
      alias: [{ find: "@", replacement: path.resolve("src") }],
    },
  },
  // timeUnit: "s",
  server: {
    port: 8854,
  },
  preview: {
    port: 3691,
  }
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
      // console.log(compiler, "compiler")
    },
  };
}
