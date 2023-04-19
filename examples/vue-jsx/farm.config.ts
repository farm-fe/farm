// change to @farmfe/core/config when resolve support conditional exports
import { defineFarmConfig } from '@farmfe/core/config';

export default defineFarmConfig({
  compilation: {
    input: {
      index: './index.html',
    },
    output: {
      path: './build',
    },
    resolve: {
      strictExports: true,
    },
  },
  plugins: [['@farmfe/plugin-vue-jsx', { name: 'erkelost' }]],
});
