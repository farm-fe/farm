import { defineConfig, loadEnv } from "@farmfe/core";

import react from "@farmfe/plugin-react";
import path from "path";

function custom() {
  return {
    name: "farm-test-plugin-name",
    buildStart: {
      executor() {
        // console.log("buildStart");
      }
    },
    config(config) {
      config.plugins.push({
        name: "test-add-plugin"
      })
      return config
    },
    resolve: {
      filters: {
        importers: ['^.*$'],
        sources: ['.*'],
      },
      executor(param) {
      }
    },
    transform: {
      filters: {
        moduleTypes: ['js'],
      },
      async executor(param, ctx) {
        // console.log(param, "transform");
      }
    },
    // renderStart: {
    //   async executor() {
    //     // update my plugin status
    //     // console.log(1231231);
    //   }
    // }
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
    // persistentCache: false,
    // persistentCache: {
    //   cacheDir: "node_modules/.adny",
    // },
    output: {
      // publicPath: "/aaa/",
    },
  },
  server: {
    port: 4855,
    appType: "mpa",
  },
});

function myCustomPlugin() {
  return {
    name: "custom",
    updateModules: {
      executor(data: any) {
        console.log(data);
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

