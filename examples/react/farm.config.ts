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
      publicPath: 'public'
    },
    presetEnv: false,
    // sourcemap: true,
    css: {
      // modules: {
      //   indentName: 'farm-[name]-[hash]'
      // },
      prefixer: {
        targets: ['last 2 versions', 'Firefox ESR', '> 1%', 'ie >= 11']
      }
    },
    treeShaking: true
  },
  server: {
    port: 3000
  },
  plugins: [
    ['@farmfe/plugin-react', { runtime: 'automatic' }],
    '@farmfe/plugin-sass'
  ],
  vitePlugins: [
    {
      name: 'vite111',
      config(config, env) {
        return config;
      },
      configResolved(config) {}
    }
    // {
    //   name: 'vite2222',
    //   config(config) {
    //     return config
    //   }
    // }
  ]
});
