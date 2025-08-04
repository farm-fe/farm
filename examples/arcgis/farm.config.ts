import { defineConfig } from "farm";
import visualizer from "@farmfe/js-plugin-visualizer";

export default defineConfig((env) => ({
  compilation: {
    // lazyCompilation: false,
    // persistentCache: false,
    // minify: env.mode === 'production' ? {
    //   exclude: [
    //     '/node_modules/@arcgis/core/',
    //   ]
    // } : false,
    persistentCache: false,
    // concatenateModules: false,
    // minify: false,
    // minify: {
    //   mangleExports: false,
    // },
    // treeShaking: false,
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
