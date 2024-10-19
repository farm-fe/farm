import { defineConfig } from "@farmfe/core";

export default defineConfig({
  compilation: {
    concatenateModules: true,
    persistentCache: false,
    treeShaking: false,
  },
  server: {
    writeToDisk: true,
  }
})