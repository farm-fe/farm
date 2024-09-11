import * as path from "path";
import * as process from "process";

import { defineConfig } from "@farmfe/core";
import vue from "@vitejs/plugin-vue";
import compression from "compression";
const compressionMiddleware = () => {
  return {
    name: "compression",
    configureServer(server) {
      server.middlewares.use(compression());
    },
  };
};
export default defineConfig({
  compilation: {
    presetEnv: {
      options: {
        targets: "Chrome >= 84",
      },
    },
    resolve: {
      alias: {
        "@/": path.join(process.cwd(), "src"),
      },
    },
    persistentCache: false,
    output: {
      filename: "static/[name].[hash].[ext]",
      assetsFilename: "static/[resourceName].[ext]",
    },
  },
  vitePlugins: [vue(), compressionMiddleware()],
});
