import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    persistentCache: false,
    input: {
      index: 'src/index.ts'
    },
    output: {
      targetEnv: 'node'
    },
    external: [
      "@napi-rs/cli"
    ],
    sourcemap: false,
    presetEnv: false,
    minify: false,
  }
});
