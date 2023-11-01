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
      publicPath: '/admin/'
    },
    sourcemap: true
  },
  server: {
    // headers: {
    //   'Access-Control-Allow-Origin': '*'
    // },
    port: 5580,
    writeToDisk: false,
    cors: true
  },
  plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass']
});
