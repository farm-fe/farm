import { defineConfig } from "@farmfe/core";
import image from "@farmfe/plugin-image"
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
    image()
  ],
});
