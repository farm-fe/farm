// change to @farmfe/core/config when resolve support conditional exports
import { defineFarmConfig } from '@farmfe/core/config';
import Sass from '@farmfe/js-plugin-sass';

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
  plugins: ['@farmfe/plugin-react', Sass()],
});
