import type { UserConfig } from '@farmfe/core';

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  compilation: {
    input: {
      index: './index.html'
    },
    output: {
      path: './build',
      publicPath: process.env.NODE_ENV === 'development' ? '/admin/' : '/'
    },
    sourcemap: true
  },
  server: {
    // headers: {
    //   'Access-Control-Allow-Origin': '*'
    // },
    port: 9200,
    hmr: true,
    writeToDisk: false,
    cors: true,
    host: '127.0.0.1'
  },
  plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass']
});
