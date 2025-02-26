import { defineConfig } from '@farmfe/core';
import farmDtsPlugin from '@farmfe/js-plugin-dts';
import path from 'node:path';
export default defineConfig({
  compilation: {
    presetEnv: false,
    progress: false,
    input: {
      index: "./index.ts"
    },
    output: {
      targetEnv: 'node'
    },
    persistentCache: false,
    resolve: {
      alias: {
        "@": path.resolve("./src")
      }
    }
  },
  // plugins: [
    // farmDtsPlugin({
      // tsConfigPath: './tsconfig.json'
    // })
  // ]
});
