import { defineConfig, loadEnv } from "@farmfe/core";
import path from 'node:path'
import react from "@farmfe/plugin-react";

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
    output: {
      publicPath: "/aaa/",
    },
  },
});
