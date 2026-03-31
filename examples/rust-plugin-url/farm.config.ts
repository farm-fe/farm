import { defineConfig } from "@farmfe/core";
import { resolve } from "path"
import url from "@farmfe/plugin-url"
import react from "@farmfe/plugin-react"
export default defineConfig({
  compilation: {
    input: {
      index: "./index.html",
    },
    persistentCache: false,
    progress: false,
  },
  plugins: [
    react({ runtime: "automatic" }),
    url({
      limit: 10 * 1024,
      publicPath: "output/",
      emitFiles: true,
      destDir: resolve(__dirname, "./dist/assets")
    })
  ],
});
