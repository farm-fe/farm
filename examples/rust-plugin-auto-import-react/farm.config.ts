import { defineConfig } from "@farmfe/core";
import react from '@farmfe/plugin-react';
import autoImport from '@farmfe/plugin-auto-import';
import visualizer from '@farmfe/js-plugin-visualizer';

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
    autoImport({
      presets: [
        "react",
        "react-router",
        "react-router-dom",
      ],
      dirs: ['src/apis'],
      ignore:[]
    }),
    visualizer()
  ],
});
