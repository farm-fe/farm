import { defineConfig } from 'farm';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
  compilation: {
    input: {
      index: './src/server.ts'
    },
    output: {
      path: './dist',
      targetEnv: 'node',
      format: 'esm'
    },
    minify: false,
    css: {
      prefixer: {
        targets: ['last 2 versions', 'Firefox ESR', '> 1%', 'ie >= 11']
      }
    },
    // partialBundling: {
    //   enforceResources: [
    //     {
    //       name: 'index',
    //       test: ['.*']
    //     }
    //   ]
    // },
    persistentCache: false,
    lazyCompilation: false
  },
  vitePlugins: [vue()]
});
