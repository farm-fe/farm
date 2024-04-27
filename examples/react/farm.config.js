import { defineConfig } from '@farmfe/core';
import path from 'path';

export default defineConfig(() => {
  // console.log(__dirname);
  // console.log(__filename);
  // console.log(__dirname);

  return {
    // root: path.resolve(process.cwd(), './html'),
    compilation: {
      sourcemap: true,
      persistentCache: false,
      presetEnv: false,
      progress: false,
      output: {
        publicPath: '/dist/'
      },
      input: {
        index: './index.html'
      }
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
