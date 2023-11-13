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
    port: 8674,
    // strictPort: true,
  },
  plugins: [farmJsPluginVue(), {
    name: "test",
    priority: 100,
    config(config) {
      // console.log(config);
      config.aaa = 123123;
    },
  }, {
    name: "wwww",
    priority: 100,
    config(config) {
      // console.log(config);
      config.bbb = 789789456146546;
    },
  }],
});
