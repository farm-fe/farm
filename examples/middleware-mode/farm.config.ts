import { defineConfig } from '@farmfe/core';

export default defineConfig({
  server: {
    middlewareMode: true,
    hmr: {
      port: 9801
    },
    middlewares
  }
});
