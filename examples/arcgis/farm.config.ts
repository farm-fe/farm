import { defineConfig } from "@farmfe/core";
import visualizer from "@farmfe/js-plugin-visualizer";

export default defineConfig((env) => ({
  compilation: {
    persistentCache: false,
    minify: env.mode === 'production' ? {
      exclude: [
        '/node_modules/@arcgis/core/',
      ]
    } : false,
    partialBundling: {
      groups: [
        {
          name: '[resourceName]',
          test: ['.*']
        }
      ]
    }
  },
  server: {
    port: 9001
  },
  plugins: [
    process.env.FARM_VISUALIZER ? visualizer() : null
  ]
}))