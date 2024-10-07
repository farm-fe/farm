import { bold, defineConfig, green, Logger, yellow } from "@farmfe/core";
import vue from "@vitejs/plugin-vue";
import { VueRouterAutoImports } from "unplugin-vue-router";
import VueRouter from "unplugin-vue-router/vite";
import AutoImport from "unplugin-auto-import/vite";
import compression from "compression";
import { federation } from "@module-federation/vite";
import { createHtmlPlugin } from "vite-plugin-html";
import viteCompression from "vite-plugin-compression";
import mkcert from "vite-plugin-mkcert";
import Inspector from "unplugin-vue-inspector/vite";
import { aaa } from "./test.js";
import path from "path";

const logger = new Logger({});
const compressionMiddleware = () => {
  return {
    name: "compression",
    configureServer(server) {
      // console.log("server", server.middlewares);
      const _printUrls = server.printUrls.bind(server);

      server.printUrls = () => {
        _printUrls();
        logger.info(
          `${green("➜")}  ${bold("Vue Inspector")}: ${green(`Press ${yellow("Ctrl(^)+Shift(⇧)")} in App to toggle the Inspector`)}\n`,
        );
      };
      server.middlewares.use(compression());
    },
  };
};

function myCustomPlugin() {
  return {
    name: "vite-plugin-custom",
    apply: "serve",
    config(config, { command }) {},
    configureServer(server) {
      server.httpServer?.once?.("listening", () => {
        const { port } = server.config.server;
      });
      server.middlewares.use((req, res, next) => {
        next();
      });
    },
    transformIndexHtml(c) {
      return c.replace(
        "<head>",
        `<head><script type="module" src=${JSON.stringify(
          "/@id/".replace(/.+?\:([/\\])[/\\]?/, "$1").replace(/\\\\?/g, "/"),
        )}></script>`,
      );
    },
    configResolved(config) {
      // console.log(config.build.rollupOptions);
    },
    // configureServer(server) {
    //   server.middlewares.use((req, res, next) => {
    //     console.log(`收到请求 之前的: ${req.url}`);
    //     next();
    //   });

    //   return () => {
    //     server.middlewares.use((req, res, next) => {
    //       console.log(`收到请求 posthook的: ${req.url}`);
    //       next();
    //     });
    //   };
    // },
  };
}

export default defineConfig({
  vitePlugins: [
    // mkcert(),
    // viteCompression(),
    // VueRouter(),
    // AutoImport({
    // imports: ["vue", VueRouterAutoImports],
    // }),
    vue(),
    // federation({
    //   name: "remote",
    //   filename: "remoteEntry.js",
    //   exposes: {
    //     "./remote-app": "./src/App.vue",
    //   },
    //   shared: ["vue"],
    // }),
    // Inspector({
    //   enabled: true,
    // }),
    // compressionMiddleware(),
    // myCustomPlugin(),
    // createHtmlPlugin({
    //   minify: true,
    //   /**
    //    * 在这里写entry后，你将不需要在`index.html`内添加 script 标签，原有标签需要删除
    //    * @default src/main.ts
    //    */
    //   // entry: 'src/main.ts',
    //   /**
    //    * 如果你想将 `index.html`存放在指定文件夹，可以修改它，否则不需要配置
    //    * @default index.html
    //    */
    //   // template: 'public/index.html',

    //   /**
    //    * 需要注入 index.html ejs 模版的数据
    //    */
    //   inject: {
    //     data: {
    //       title: 'index',
    //       injectScript: `<script src="./src/index.ts"></script>`,
    //     },
    //     tags: [
    //       {
    //         injectTo: 'body-prepend',
    //         tag: 'div',
    //         attrs: {
    //           id: 'tag',
    //         },
    //       },
    //     ],
    //   },
    // }),
    // Inspect(),
    // compressionMiddleware(),
  ],
  // customLogger: logger,

  compilation: {
    input: {
      index: "index.html",
    },
    persistentCache: false,
    runtime: {
      isolate: true,
    },
    progress: false,
    resolve: {
      // alias: {
        // "@": path.resolve("src"),
      // },
      alias: [{ find: "@", replacement: path.resolve("src") }],
    },
  },
  server: {
    // https: true,
    port: 5380,
  },
  // plugins: [myCustomPlugin()],
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
