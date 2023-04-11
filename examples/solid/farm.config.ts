// change to @farmfe/core/config when resolve support conditional exports
import { defineFarmConfig } from '@farmfe/core/dist/config';
import solid from '@farmfe/js-plugin-solid';

export default defineFarmConfig({
  compilation: {
    treeShaking: false,
    input: {
      index: './index.html',
    },
    output: {
      path: './build',
    },
  },
  plugins: [solid()],
});
