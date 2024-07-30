import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    persistentCache: false,
    progress: false
  },
  clearScreen: false,
  server: {
    host: '127.0.0.1'
  }
});
