import { defineConfig } from '@farmfe/core';

export default defineConfig(() => {
  return {
    compilation: {
      sourcemap: true,
      // persistentCache: false,
      presetEnv: false,
      progress: false
      // output: {
      //   publicPath: '/dist/'
      // },
    },
    server: {
      port: 4000,
      proxy: {
        '^/(api|login|register|messages)': {
          target: 'https://petstore.swagger.io/v2',
          ws: true
        },
      }
    },
    plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass']
  };
});
