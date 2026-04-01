import { defineConfig } from "@farmfe/core";
import farmJsPluginPostcss from '@farmfe/js-plugin-postcss';
import visualizer from '@farmfe/js-plugin-visualizer'
import icons from "@farmfe/plugin-icons"
import react from "@farmfe/plugin-react"
export default defineConfig({
  compilation: {
    input: {
      index: "./index.html",
    },
    persistentCache: true,
    progress: false,
  },
  plugins: [
    farmJsPluginPostcss(),
    visualizer(),
    react(),
    icons({
      scale: 1.2,
      autoInstall: true,
      compiler: "jsx",
      defaultClass: "icon-color",
      customCollections: {
        local: './src/assets',
        remote: "https://cdn.simpleicons.org/[iconname]"
      }
    }),
  ],
});
