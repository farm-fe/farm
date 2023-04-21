import type { UserConfig } from '@farmfe/core';

export default <UserConfig>{
  compilation: {
    input: {
      index: './index.html',
    },
    resolve: {
      symlinks: true,
    },
    define: {
      BTN: 'Click me',
      HMR_PORT: 8008,
    },
    output: {
      path: './build',
    },
    sourcemap: true,
  },
  server: {
    hmr: {
      port: 3001,
    },
  },
  plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass'],
};
