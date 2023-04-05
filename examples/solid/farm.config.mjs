// change to @farmfe/core/config when resolve support conditional exports
import { defineFarmConfig } from '@farmfe/core/config';
import solid from '@farmfe/js-plugin-solid';

export default defineFarmConfig({
  compilation: {
    input: {
      index: './index.html',
    },
    output: {
      path: './build',
    },
  },
  plugins: [solid()],
});
