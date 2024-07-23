import { defineConfig } from '@farmfe/core';

export default defineConfig({
  plugins: ['@farmfe/plugin-react'],
  compilation: {
    presetEnv: false,
    progress: false,
    sourcemap: false,
    runtime: {
      isolate: true
    }
  }
});
