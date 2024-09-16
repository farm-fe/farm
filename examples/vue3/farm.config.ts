import { defineConfig, Logger } from "@farmfe/core";
import vue from "@vitejs/plugin-vue";
// import { VueRouterAutoImports } from "unplugin-vue-router";
// import VueRouter from "unplugin-vue-router/vite";
// import AutoImport from 'unplugin-auto-import/vite';
import compression from "compression";
import { createHtmlPlugin } from 'vite-plugin-html'
import viteCompression from 'vite-plugin-compression';
import mkcert from 'vite-plugin-mkcert'
import Inspect from 'vite-plugin-inspect'
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

function myCustomPlugin() {
  return {
    name: 'vite-plugin-custom',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        // console.log(`收到请求: ${req.url}`);
        next();
      });
      return () => {
        server.middlewares.use((req, res, next) => {
          // console.log(`收到请求: ${req.url}`);
          next();
        });
      };
    },
    configResolved(resolvedConfig) {
      // console.log(resolvedConfig.env);
    },
  }
}


const logger = new Logger({
  prefix: "我是曼"
});


export default defineConfig({
  vitePlugins: [
    // mkcert(),
    viteCompression(),
    // VueRouter(),
    // AutoImport({
    //   imports: ["vue", VueRouterAutoImports],
    // }),
    vue(),
    myCustomPlugin(),
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
    persistentCache: false,
  },
  server: {
    // https: true,
    port: 5384,
  },
  plugins: [
    // myCustomPlugin(),
  ]
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
