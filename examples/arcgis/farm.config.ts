import { defineConfig } from "@farmfe/core";
import visualizer from "@farmfe/js-plugin-visualizer";

export default defineConfig((env) => ({
  compilation: {
    // persistentCache: false,
    minify: env.mode === 'production' ? {
      exclude: [
        '/node_modules/@arcgis/core/',
      ]
    } : false,
    partialBundling: {
      enforceTargetConcurrentRequests: false,
      enforceTargetMinSize: true,
      targetMinSize: 1024 * 200,
      enforceResources: [
        {
          name: 'index',
          test: ['^src/.+']
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