import { defineConfig } from "@farmfe/core";
import react from '@farmfe/plugin-react';
import farmPlugin from '@farmfe/plugin-compress';

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
    farmPlugin({
      algorithm: 'brotli',
      filter: '\\.(js|mjs|json|css|html)$',
      level: 11,
      threshold: 2048,
      deleteOriginFile: true,
    })
  ],
});
