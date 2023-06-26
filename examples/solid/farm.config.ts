import { defineFarmConfig } from '@farmfe/core/config';
import farmJsPluginSolid from '@farmfe/js-plugin-solid';

export default defineFarmConfig({
  compilation: {
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
  plugins: [farmJsPluginSolid()]
});
