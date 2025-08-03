import { defineConfig } from "farm";
import react from '@farmfe/plugin-react';
import farmPlugin from '<FARM-RUST-PLUGIN-NPM-NAME>';

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
    farmPlugin()
  ],
});
