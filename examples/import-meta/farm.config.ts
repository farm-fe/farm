import { defineConfig } from "@farmfe/core";

export default defineConfig({
  compilation: {
    input: {
      index: "./index.ts",
    },
    output: {
      targetEnv: 'node',
    },
    minify: false,
    persistentCache: false
  }
})