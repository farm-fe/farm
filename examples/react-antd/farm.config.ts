import type { UserConfig } from "@farmfe/core";

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  compilation: {
    input: {
      index: "./index.html",
    },
    output: {
      path: "./build",
      publicPath: "/admin/",
    },
    sourcemap: false,
    persistentCache: true,
  },
  server: {
    // headers: {
    //   'Access-Control-Allow-Origin': '*'
    // },
    writeToDisk: false,
    cors: true,
    hmr: {
      port: 6542
    }
  },
  plugins: ["@farmfe/plugin-react", "@farmfe/plugin-sass"],
});
