import type { UserConfig } from '@farmfe/core';

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
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
      path: './build',
      publicPath: 'public'
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
    treeShaking: true,
    persistentCache: false
  },
  server: {
    cors: true
  },
  plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass'],
  vitePlugins: [
    // {
    //   name: 'vite-plugin-test2',
    //   config(config, env) {
    //     config.b = 123;
    //   }
    // },
    // {
    //   name: 'vite-plugin-test3',
    //   config(config, env) {
    //     config.c = 123;
    //   },
    //   configResolved(config) {
    //     console.log(config);
    //   }
    // }
  ]
});
