import { defineConfig } from '@farmfe/core';
import farmDtsPlugin from '@farmfe/js-plugin-dts';
export default defineConfig({
  compilation: {
    presetEnv: false,
    progress: false,
    input: {
      index: "./index.ts"
    },
    persistentCache: false
  },
  // plugins: [
  //   farmDtsPlugin({
  //     tsConfigPath: './tsconfig.json'
  //   })
  // ]
  plugins: ['@farmfe/plugin-dts']
});
