import farmDtsPlugin from '@farmfe/js-plugin-dts';

/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
  compilation: {
    input: {
      index: './index.ts'
    },
    output: {
      path: 'dist',
      filename: 'index.[ext]'
    }
  },
  server: {
    hmr: false
  },
  plugins: [farmDtsPlugin()]
};
