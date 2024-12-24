import { defineConfig } from "@farmfe/core";
// import Vue from "unplugin-vue/farm";
import Vue from "unplugin-vue/vite";
import fs from 'fs'

export default defineConfig({
  // plugins: [Vue(),base()],
  vitePlugins: [Vue()],
  compilation: {
    progress:false,
    persistentCache: false,
  },
  server: {
    origin: "http://localhost:5425"
  }
});

function base() {
  return {
    name: "farm-load-vue-module-type",
      priority: -100,
      load: {
        filters: {
          resolvedPaths: [".vue"],
        },
        executor: (param) => {
          const content = fs.readFileSync(param.resolvedPath, "utf-8");
          return {
            content,
            moduleType: "js",
          };
        },
      },
  };
}

