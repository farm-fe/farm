import { defineConfig } from "@farmfe/core";

export default defineConfig({
  compilation: {
    input: {
      index: "./index.cjs"
    },
    output: {
      targetEnv: 'node'
    },
    sourcemap: false,
    minify: false,
    external: ['@farmfe/core']
  }
})
