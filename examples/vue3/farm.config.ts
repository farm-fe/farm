import { defineConfig, Logger } from "@farmfe/core";
import vue from "@vitejs/plugin-vue";
// import { VueRouterAutoImports } from "unplugin-vue-router";
// import VueRouter from "unplugin-vue-router/vite";
// import AutoImport from 'unplugin-auto-import/vite';
import compression from "compression";
import { aaa } from "./test.js"
const compressionMiddleware = () => {
  return {
    name: "compression",
    configureServer(server) {
      // console.log("server", server.middlewares);
      server.middlewares.use(compression());
    }
  };
};

const logger = new Logger({
  prefix: "我是曼"
});


export default defineConfig({
  vitePlugins: [
    // VueRouter(),
    // AutoImport({
    //   imports: ["vue", VueRouterAutoImports],
    // }),
    vue(),
    compressionMiddleware(),
  ],
  // customLogger: logger,

  compilation: {
    // persistentCache: false,
  },
  server: {
    port: 5384,
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
