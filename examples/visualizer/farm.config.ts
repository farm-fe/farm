import { defineConfig } from "@farmfe/core";
// import viewer from "@farmfe/js-plugin-visualizer";
import react from "@farmfe/plugin-react";
import compression from "compression";
const myPlugin = () => ({
  name: "configure-server",
  configureServer(server) {
    server.middlewares.use(compression());
    server.middlewares.use((req, res, next) => {
      // 自定义请求处理...
      next();
    });
  },
});

export default defineConfig({
  plugins: [react(), myPlugin()],
  compilation: {
    output: {
      // publicPath: "/aaa/",
    },
  },
  server: {
    // port: 3000,
  },
});
