import { defineConfig } from 'farm';
import farmJsPluginSolid from '@farmfe/js-plugin-solid';

export default defineConfig({
  compilation: {
    minify: false,
    presetEnv: false,
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
    // open: true,
    port: 6270,
  },
  plugins: [farmJsPluginSolid()]
});
