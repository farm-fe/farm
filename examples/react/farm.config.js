import { defineConfig } from '@farmfe/core';

export default defineConfig(() => {
  return {
    compilation: {
      sourcemap: true,
      persistentCache: false,
      presetEnv: false,
      progress: false,
      // output: {
      //   publicPath: '/dist/'
      // },
    },
    server: {
      port: 6532,
      hmr: {
        path: '/__farm_hmr'
      }
    },
    plugins: [
      '@farmfe/plugin-react',
      '@farmfe/plugin-sass'
    ]
  };
});
