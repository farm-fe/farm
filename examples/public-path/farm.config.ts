import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    input: {
      index: './index.html'
    },
    resolve: {
      symlinks: true
    },
    output: {
      path: './build',
      publicPath: '/public-path/'
    },
    // sourcemap: true,
    css: {
      // modules: {
      //   indentName: 'farm-[name]-[hash]'
      // },
      prefixer: {
        targets: ['last 2 versions', 'Firefox ESR', '> 1%', 'ie >= 11']
      }
    },
    persistentCache: false,
    // treeShaking: true,
    minify: false
  },
  server: {
    cors: true,
    port: 6260,
    host: 'localhost'
  },
  plugins: [
    '@farmfe/plugin-react',
  ]
});
