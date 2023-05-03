import type { UserConfig } from '@farmfe/core';
import farmLessPlugin from '@farmfe/js-plugin-less';

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
    sourcemap: false
    // treeShaking: true,
    // minify: true,
  },
  server: {
    hmr: true,
  },
  plugins: ['@farmfe/plugin-react',farmLessPlugin({
    additionalData: `@hoverColor: #f10215;`
  }) ],
};
