import { defineConfig } from '@farmfe/core';
import vue from '@vitejs/plugin-vue';
import icons from "@farmfe/plugin-icons"
export default defineConfig({
  compilation: {
    input: {
      index: "./index.html",
    },
    persistentCache: false,
    progress: false,
  },
  vitePlugins: [vue()],
  plugins: [
    icons({
      autoInstall: true,
      compiler: 'vue',
      // defaultStyle: {
      //   width: "2em",
      //   height: "2em",
      // },
      customCollections: {
        local: './src/assets',
        remote: "https://cdn.simpleicons.org/[iconname]/"
      }
    }),
  ]
});
