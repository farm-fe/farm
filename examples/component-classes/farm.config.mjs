import { defineConfig } from "@farmfe/core";

export default defineConfig({
  compilation: {
    input: {
      index: "./index.js",
    },
    minify: false,
    output: {
      entryFilename: "[entryName].mjs",
    },
  },
});
