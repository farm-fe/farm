import { defineConfig } from "@farmfe/core";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  vitePlugins: [vue()],
  server: {
    port: 5232,
  },
});
