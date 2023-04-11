import type { UserConfig } from '@farmfe/core';

export default <UserConfig>{
  compilation: {
    input: {
      index: './index.html'
    },
    resolve: {
      symlinks: true
    },
    define: {
      BTN: 'Click me'
    },
    output: {
      path: './build'
    },
    sourcemap: true
  },
  server: {
    hmr: true
  },
  plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass']
};
