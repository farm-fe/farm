// change to @farmfe/core/config when resolve support conditional exports
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
    define: {
      BTN: 'Click me',
    },
    output: {
      path: './build',
    },
  },
  server: {
    hmr: true,
  },
});
