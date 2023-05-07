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
    },
    output: {
      path: './build',
    },
    sourcemap: false,
    css: {
      modules: {
        indentName: 'farm-[name]-[hash]',
      },
      prefixer: {
        targets: {
          chrome: '58',
          ie: '11',
        },
      },
    },
    // treeShaking: true,
    // minify: true,
  },
  server: {
    hmr: true,
  },
  plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass'],
};
