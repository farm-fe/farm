import { defineConfig } from '@farmfe/core';

export default defineConfig({
  plugins: ['@farmfe/plugin-react'],
  compilation: {
    presetEnv: false,
    minify: false,
    progress: false,
    sourcemap: false,
    // persistentCache: false,
    runtime: {
      isolate: true
    },
    output: {
      publicPath: '/dist'
    }
  },
  server: {
    port: 3212,
    // writeToDisk: true,
  }
});
