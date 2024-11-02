import { defineConfig } from "@farmfe/core";

import react from "@farmfe/plugin-react";
import path from "path";
// load: {
//   // filters: {
//   //   resolvedPaths: ['\\.*$'] // filter files to improve performance
//   // },
//   async executor({ resolvedPath }) {
//     console.log(resolvedPath);
//   }
// }

function custom() {
  return {
    name: "farm-test-plugin-name",

    transform: {
      filters: {
        // moduleTypes: ['js'],
      },
      async executor(param, ctx) {
        // console.log(param, "transform");
      }
    }
  }
}


export default defineConfig({
  plugins: [
    react(),
    // myCustomPlugin(),
    // compilerPlugin(),
    custom()
  ],
  compilation: {
    input: {
      index: path.resolve(__dirname, "index.html"),
      base: path.resolve(__dirname, 'base.html'),
      about: path.resolve(__dirname, 'about.html'),
    },
    progress: false,
    persistentCache: false,
    // persistentCache: {
    //   cacheDir: "node_modules/.adny",
    // },
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

