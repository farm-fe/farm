// change to @farmfe/core/config when resolve support conditional exports
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
  plugins: ['@farmfe/plugin-react'],
});
