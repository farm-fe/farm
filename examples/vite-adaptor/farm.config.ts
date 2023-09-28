import type { UserConfig } from "@farmfe/core";
import farmJsPluginVue from "@farmfe/js-plugin-vue";

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
    },
    resolve: {
      strictExports: true,
    },
  },
  server: {
    port: 6654,
  },
  plugins: [farmJsPluginVue({ framework: "farm" })],
});
