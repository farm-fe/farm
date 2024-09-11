import { defineConfig } from "@farmfe/core";
import vue from "@vitejs/plugin-vue";
// import { VueRouterAutoImports } from "unplugin-vue-router";
// import VueRouter from "unplugin-vue-router/vite";
// import AutoImport from 'unplugin-auto-import/vite';
import compression from "compression";
const compressionMiddleware = () => {
  return {
    name: "compression",
    configureServer(server) {
      // console.log("server", server.middlewares);
      server.middlewares.use(compression());
    },
  };
};


export default defineConfig({
  vitePlugins: [
    // VueRouter(),
    // AutoImport({
    //   imports: ["vue", VueRouterAutoImports],
    // }),
    vue(),
    compressionMiddleware(),
  ],

  compilation: {
    // persistentCache: false,
  },
  server: {
    port: 5244,
  },

  // plugins: [compressionMiddleware()],
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
