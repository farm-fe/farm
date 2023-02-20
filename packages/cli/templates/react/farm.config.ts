import { defineFarmConfig } from '@farmfe/core/dist/config';

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
