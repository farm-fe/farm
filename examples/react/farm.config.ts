import type { UserConfig } from "@farmfe/core";

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  plugins: [
    "@farmfe/plugin-react",
    "@farmfe/plugin-sass",
  ],
  vitePlugins: [
    {
      name: "virtual-module",
      config(config) {
        console.log(config);
        config.a = 123123;
      },
    },
    {
      name: "virtual-module2",
      config(config) {
        console.log(config);
        config.b = 456456;
      },
    },
  ],
});
