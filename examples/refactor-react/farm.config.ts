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
    input: {
      index: path.resolve(__dirname, "index.html"),
      base: path.resolve(__dirname, 'base.html'),
      about: path.resolve(__dirname, 'about.html'),
    },
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
    // appType: "mpa",
  },
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
