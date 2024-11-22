import { defineConfig } from "@farmfe/core";
import react from '@farmfe/plugin-react';

export default defineConfig({
  compilation: {
    concatenateModules: true,
    persistentCache: false,
    treeShaking: false,
  },
  server: {
    writeToDisk: true,
  },
  plugins: [react()]
})