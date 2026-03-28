import { defineConfig } from '@farmfe/core';
import strip from "@farmfe/plugin-strip";
import TanStackRouterVite from "@tanstack/router-plugin/vite"
export default defineConfig({
  plugins: ['@farmfe/plugin-react',
    strip({
      functions: ['console.*', 'assert.*'],
      labels: ['unittest']
    })
  ],
  compilation: {
    minify: false,
    persistentCache: false,
  },
  vitePlugins: [
    TanStackRouterVite({
      target: 'react',
      routesDirectory: 'src/routes/',
      autoCodeSplitting: false,
    }),
  ]
});
