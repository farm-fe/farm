import { defineConfig, loadEnv } from "@farmfe/core";

import react from "@farmfe/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [
    react(),
  ],
  compilation: {
    input: {
      index: path.resolve(__dirname, "index.html"),
      base: path.resolve(__dirname, 'base.html'),
      about: path.resolve(__dirname, 'about.html'),
    },
    progress: false,
    // persistentCache: false,
    output: {
    },
  },
});
