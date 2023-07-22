import farmJsPluginSolid from '@farmfe/js-plugin-solid';

export default {
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
};
