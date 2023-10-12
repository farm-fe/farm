import type { UserConfig } from '@farmfe/core';
import solid from 'vite-plugin-solid';

function defineFarmConfig(config: UserConfig) {
  return config;
}

export default defineFarmConfig({
  compilation: {
    input: {
      index: 'index.html'
    },
    output: {
      path: 'build'
    },
    define: {
      __DEV__: 'true'
    },
    sourcemap: false,
    resolve: {
      alias: {
        'solid-js$': 'solid-js/dist/dev.js',
        'solid-js/web$': 'solid-js/web/dist/dev.js',
      }
    },
  },
  server: {
    open: true
  },
  vitePlugins: [solid()]
});
