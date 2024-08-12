import { defineConfig } from "@farmfe/core";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  vitePlugins: [vue()],
  server: {
    port: 5232,
    proxy: {
      "/aaa": {
        target: "http://v.juhe.cn/toutiao/index",
        changeOrigin: true,
        ws: true,
        rewrite: (path) => path.replace(/^\/aaa/, ""),
      },
    },
  },
});
