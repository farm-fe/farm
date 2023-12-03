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
      // publicPath: '/public/'
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
    port: 3652,
    open: true
  },
  plugins: [
    ['@farmfe/plugin-react', { runtime: 'automatic' }],
    '@farmfe/plugin-sass',
    {
      name: 'plugin-finish-hook-test',
      finish: {
        executor(param, context, hookContext) {
          // console.log('plugin-finish-hook-test', param, context, hookContext);
        }
      }
    },
    {
      name: 'plugin-hook-context-test',
      load: {
        filters: {
          resolvedPaths: ['.+main.tsx']
        },
        executor(param, context, hookContext) {
          // console.log('plugin-hook-context-test', param, context, hookContext);
          // console.log(context.getWatchFiles());
          // context.emitFile({
          //   resolvedPath: param.resolvedPath,
          //   name: "test.txt",
          //   // Buffer to number[]

          //   content: [...Buffer.from("test")],
          //   resourceType: "txt"
          // });
          // context.addWatchFile(param.resolvedPath, path.join(process.cwd(), 'src', 'original-sourcemap', 'config.d.ts'));
          // context.warn('test');
          // context.error('test');
          return null;
        }
      }
    },
    {
      name: 'plugin-update-modules-hook-test',
      updateModules: {
        executor(param, context, hookContext) {
          // console.log("params", param);
          // console.log("context", context);
          // console.log("hookContext", hookContext);
        }
      }
    }
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
