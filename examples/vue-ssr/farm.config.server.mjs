import vue from '@vitejs/plugin-vue';

/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
  compilation: {
    input: {
      index: './src/server.ts'
    },
    output: {
      path: './dist',
      targetEnv: 'node',
      format: 'esm'
    },
    minify: false,
    css: {
      prefixer: {
        targets: ['last 2 versions', 'Firefox ESR', '> 1%', 'ie >= 11']
      }
    },
    persistentCache: false,
    lazyCompilation: false
  },
  vitePlugins: [vue()]
};
