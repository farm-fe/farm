import { defineConfig } from "farm";

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
