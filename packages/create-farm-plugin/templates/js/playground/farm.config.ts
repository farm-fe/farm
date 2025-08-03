import { defineConfig } from "farm";

export default defineConfig({
  compilation: {
    input: {
      index: "./index.html",
    },
    persistentCache: false,
    progress: false,
  },
  plugins: [
    ["@farmfe/plugin-react", { runtime: "automatic" }],
  ],
});
