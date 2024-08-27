import { defineConfig } from "@farmfe/core";
import vue from "@vitejs/plugin-vue";
// import { VueRouterAutoImports } from "unplugin-vue-router";
// import VueRouter from "unplugin-vue-router/vite";
// import AutoImport from 'unplugin-auto-import/vite';

export default defineConfig({
  vitePlugins: [
    // VueRouter(),
    // AutoImport({
    //   imports: ["vue", VueRouterAutoImports],
    // }),
    vue(),
  ],
  // compilation: {
  //   persistentCache: false,
  // },
  // server: {
  //   port: 5232,
  //   proxy: {
  //     "/aaa": {
  //       target: "http://v.juhe.cn/toutiao/index",
  //       changeOrigin: true,
  //       ws: true,
  //       rewrite: (path) => path.replace(/^\/aaa/, ""),
  //     },
  //   },
  // },
});
