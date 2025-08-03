import { defineConfig } from "farm";

export default defineConfig({
  compilation: {
    persistentCache: false,
    progress: false
  },
  server: {
    host: '127.0.0.1'
  }
});
