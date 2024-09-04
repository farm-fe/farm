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
      console.log("Middleware stack:", server.middlewares.stack.length);
      // console.log("server", server.middlewares);
      server.middlewares.use(compression());
      server.middlewares.use((req, res, next) => {
        next();
      });
      console.log("Middleware count:", server.middlewares.stack.length);
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
