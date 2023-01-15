// change to @farmfe/core/config when resolve support conditional exports
import { defineFarmConfig } from '@farmfe/core/dist/node/config';

export default defineFarmConfig({
  compilation: {
    input: {
      index: './index.ts',
    },
    output: {
      path: 'dist',
    },
  },
});
