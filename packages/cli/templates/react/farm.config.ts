import { defineFarmConfig } from '@farmfe/core/dist/node/config';

export default defineFarmConfig({
  compilation: {
    input: {
      index: './index.html',
    },
    resolve: {
      symlinks: true,
      mainFields: ['module', 'main', 'customMain'],
    },
    output: {
      path: './build',
    },
  },
  server: {
    hmr: true,
  },
});
