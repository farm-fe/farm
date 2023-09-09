import type { UserConfig } from '@farmfe/core';
import farmJsPluginSolid from '@farmfe/js-plugin-solid';

function defineFarmConfig(config: UserConfig) {
  return config;
}

export default defineFarmConfig({
  compilation: {
    minify: false,
    presetEnv: false,
    mode: process.env.NODE_ENV,
    input: {
      index: 'index.html'
    },
    output: {
      path: 'build'
    },
    define: {
      __DEV__: 'true'
    }
  },
  server: {
    // open: true
  },
  plugins: [farmJsPluginSolid()]
});
