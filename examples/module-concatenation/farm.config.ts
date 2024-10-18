import { defineConfig } from "@farmfe/core";

export default defineConfig({
  compilation: {
    persistentCache: false,
    treeShaking: false,
  }
})