import type { UserConfig } from '@farmfe/core';

export default <UserConfig>{
  compilation: {
    input: {
      index: './index.html',
    },
    output: {
      path: './build',
    },
    sourcemap: false
    // treeShaking: true,
    // minify: true,
  },
  server: {
    hmr: true,
  },
  plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass'],
};
